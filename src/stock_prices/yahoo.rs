use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{ACCEPT, ORIGIN, REFERER};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;

const QUOTE_URL: &str = "https://query2.finance.yahoo.com/v7/finance/quote";
const COOKIE_URL: &str = "https://finance.yahoo.com/quote/AAPL";
const CRUMB_URL: &str = "https://query1.finance.yahoo.com/v1/test/getcrumb";
const USER_AGENT: &str = concat!("Mozilla/5.0 (compatible; stock-prices/", env!("CARGO_PKG_VERSION"), ")");
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, thiserror::Error)]
pub enum YahooError {
    #[error("yahoo finance request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("could not obtain a yahoo finance crumb: {0}")]
    Crumb(String),
    #[error("yahoo finance returned an error: {0}")]
    Api(String),
    #[error("yahoo finance returned {status}: {body}")]
    Status { status: StatusCode, body: String },
    #[error("unexpected yahoo finance response: {0}")]
    Unexpected(String),
}

/// A single entry of Yahoo's `v7/finance/quote` response, before normalization.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YahooQuote {
    pub symbol: String,
    pub quote_type: Option<String>,
    pub long_name: Option<String>,
    pub market: Option<String>,
    pub regular_market_price: Option<f64>,
    pub regular_market_change: Option<f64>,
    pub regular_market_change_percent: Option<f64>,
    pub regular_market_day_high: Option<f64>,
    pub regular_market_day_low: Option<f64>,
    pub regular_market_open: Option<f64>,
    pub regular_market_time: Option<i64>,
    pub regular_market_previous_close: Option<f64>,
    pub pre_market_price: Option<f64>,
    pub pre_market_change: Option<f64>,
    pub pre_market_change_percent: Option<f64>,
    pub pre_market_time: Option<i64>,
    pub post_market_price: Option<f64>,
    pub post_market_change: Option<f64>,
    pub post_market_change_percent: Option<f64>,
    pub post_market_time: Option<i64>,
    #[serde(rename = "forwardPE")]
    pub forward_pe: Option<f64>,
    pub price_to_book: Option<f64>,
    pub dividend_yield: Option<f64>,
}

/// Minimal Yahoo Finance quote client.
///
/// Yahoo gates `v7/finance/quote` behind a session cookie plus a matching
/// "crumb" token, so the first call seeds both and later calls reuse them until
/// Yahoo rejects them.
pub struct YahooClient {
    session: RwLock<Arc<YahooSession>>,
    endpoints: Endpoints,
}

struct YahooSession {
    http: Client,
    crumb: Option<String>,
}

impl YahooSession {
    fn new() -> Result<Self, YahooError> {
        // Keep cookie jars separate so refreshing cannot change an in-flight
        // request's cookie/crumb pair. Yahoo's edge requires HTTP/1.1.
        let http = Client::builder().user_agent(USER_AGENT).cookie_store(true).http1_only().timeout(REQUEST_TIMEOUT).build()?;
        Ok(Self { http, crumb: None })
    }
}

struct Endpoints {
    quote: String,
    cookie: String,
    crumb: String,
}

impl YahooClient {
    pub fn new() -> Result<Self, YahooError> {
        Ok(Self {
            session: RwLock::new(Arc::new(YahooSession::new()?)),
            endpoints: Endpoints {
                quote: QUOTE_URL.to_owned(),
                cookie: COOKIE_URL.to_owned(),
                crumb: CRUMB_URL.to_owned(),
            },
        })
    }

    /// Fetches quotes for `symbols`, retrying once with a fresh crumb when
    /// Yahoo rejects the cached one.
    pub async fn quote(&self, symbols: &[String], lang: &str, region: &str) -> Result<Vec<YahooQuote>, YahooError> {
        let joined = symbols.join(",");
        let session = self.session(None).await?;

        match self.quote_once(&joined, lang, region, &session).await {
            Err(error) if is_stale_credentials(&error) => {
                let refreshed = self.session(Some(&session)).await?;
                self.quote_once(&joined, lang, region, &refreshed).await
            }
            result => result,
        }
    }

    async fn quote_once(&self, symbols: &str, lang: &str, region: &str, session: &YahooSession) -> Result<Vec<YahooQuote>, YahooError> {
        let crumb = session.crumb.as_deref().ok_or_else(|| YahooError::Crumb("session was not initialized".to_owned()))?;
        let response = session
            .http
            .get(&self.endpoints.quote)
            .query(&[("symbols", symbols), ("lang", lang), ("region", region), ("crumb", crumb)])
            .header(ACCEPT, "application/json")
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        // Authentication status must survive even when JSON has an unrelated
        // description such as "Forbidden".
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            return Err(YahooError::Status { status, body });
        }
        let payload: Value = serde_json::from_str(&body).map_err(|error| {
            if status.is_success() {
                YahooError::Unexpected(format!("response was not valid JSON: {error}"))
            } else {
                YahooError::Status { status, body: body.clone() }
            }
        })?;

        if let Some(description) = api_error(&payload) {
            return Err(YahooError::Api(description));
        }

        if !status.is_success() {
            return Err(YahooError::Status { status, body });
        }

        let envelope: QuoteEnvelope = serde_json::from_value(payload).map_err(|error| YahooError::Unexpected(error.to_string()))?;
        let results = envelope
            .quote_response
            .and_then(|quote_response| quote_response.result)
            .ok_or_else(|| YahooError::Unexpected("quoteResponse.result was missing".to_owned()))?;

        // Delisted symbols come back as quoteType "NONE"; drop them so callers
        // only ever see tradeable instruments.
        Ok(results.into_iter().filter(|quote| quote.quote_type.as_deref() != Some("NONE")).collect())
    }

    async fn session(&self, rejected: Option<&Arc<YahooSession>>) -> Result<Arc<YahooSession>, YahooError> {
        {
            let cached = self.session.read().await;
            if session_is_usable(&cached, rejected) {
                return Ok(cached.clone());
            }
        }

        let mut cached = self.session.write().await;
        // Another request may have replaced the rejected session while this
        // request waited. Reuse that replacement instead of refreshing again.
        if session_is_usable(&cached, rejected) {
            return Ok(cached.clone());
        }
        let session = Arc::new(self.create_session().await?);
        *cached = session.clone();
        Ok(session)
    }

    async fn create_session(&self) -> Result<YahooSession, YahooError> {
        let mut session = YahooSession::new()?;

        // This request exists purely for its `set-cookie` headers, which the
        // client's cookie store keeps for the crumb and quote calls.
        session
            .http
            .get(&self.endpoints.cookie)
            .header(ACCEPT, "text/html,application/xhtml+xml,application/xml")
            .send()
            .await?;

        let response = session
            .http
            .get(&self.endpoints.crumb)
            .header(ACCEPT, "*/*")
            .header(ORIGIN, "https://finance.yahoo.com")
            .header(REFERER, &self.endpoints.cookie)
            .send()
            .await?;

        let status = response.status();
        let crumb = response.text().await?.trim().to_owned();

        if !status.is_success() {
            return Err(YahooError::Crumb(format!("crumb endpoint returned {status}")));
        }

        if crumb.is_empty() || crumb.contains('<') {
            return Err(YahooError::Crumb("crumb endpoint returned no token".to_owned()));
        }

        session.crumb = Some(crumb);
        Ok(session)
    }
}

fn session_is_usable(cached: &Arc<YahooSession>, rejected: Option<&Arc<YahooSession>>) -> bool {
    cached.crumb.is_some() && rejected.is_none_or(|rejected| !Arc::ptr_eq(cached, rejected))
}

#[derive(Debug, Deserialize)]
struct QuoteEnvelope {
    #[serde(rename = "quoteResponse")]
    quote_response: Option<QuoteResponse>,
}

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    #[serde(default)]
    result: Option<Vec<YahooQuote>>,
}

/// A cached cookie/crumb pair eventually expires; Yahoo then answers with an
/// auth status or an "Invalid Crumb" style description. Only those are worth a
/// retry — a bad symbol list would fail again just as fast.
fn is_stale_credentials(error: &YahooError) -> bool {
    match error {
        YahooError::Status { status, .. } => matches!(*status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN),
        YahooError::Api(description) => {
            let description = description.to_lowercase();
            description.contains("crumb") || description.contains("unauthorized") || description.contains("cookie")
        }
        _ => false,
    }
}

/// Yahoo reports failures as `{<single key>: {error: {code, description}}}`,
/// where the key varies by endpoint (`quoteResponse`, `finance`, ...).
fn api_error(payload: &Value) -> Option<String> {
    let object = payload.as_object()?;

    for value in object.values() {
        let Some(error) = value.get("error") else {
            continue;
        };
        if error.is_null() {
            continue;
        }

        let description = error
            .get("description")
            .and_then(Value::as_str)
            .or_else(|| error.get("code").and_then(Value::as_str))
            .unwrap_or("unknown error");

        return Some(description.to_owned());
    }

    None
}

#[cfg(test)]
mod http_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_error_descriptions_from_any_top_level_key() {
        let payload = serde_json::json!({"finance": {"result": null, "error": {"code": "Unauthorized", "description": "Invalid Crumb"}}});

        assert_eq!(api_error(&payload), Some("Invalid Crumb".to_owned()));
    }

    #[test]
    fn ignores_null_errors() {
        let payload = serde_json::json!({"quoteResponse": {"result": [], "error": null}});

        assert_eq!(api_error(&payload), None);
    }

    #[test]
    fn parses_quote_nodes() {
        let payload = serde_json::json!({
            "quoteResponse": {
                "result": [
                    {"symbol": "AAPL", "quoteType": "EQUITY", "longName": "Apple Inc.", "market": "us_market", "regularMarketPrice": 230.12, "forwardPE": 28.5},
                    {"symbol": "DEAD", "quoteType": "NONE"},
                ],
                "error": null,
            },
        });

        let envelope: QuoteEnvelope = serde_json::from_value(payload).expect("envelope parses");
        let results = envelope.quote_response.and_then(|quote_response| quote_response.result).expect("result present");
        let live: Vec<YahooQuote> = results.into_iter().filter(|quote| quote.quote_type.as_deref() != Some("NONE")).collect();

        assert_eq!(live.len(), 1);
        assert_eq!(live[0].symbol, "AAPL");
        assert_eq!(live[0].long_name.as_deref(), Some("Apple Inc."));
        assert_eq!(live[0].market.as_deref(), Some("us_market"));
        assert_eq!(live[0].forward_pe, Some(28.5));
    }
}
