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
    http: Client,
    crumb: RwLock<Option<String>>,
}

impl YahooClient {
    pub fn new() -> Result<Self, YahooError> {
        // Yahoo's edge aborts our HTTP/2 streams with a protocol error, so pin
        // the connection to HTTP/1.1.
        let http = Client::builder().user_agent(USER_AGENT).cookie_store(true).http1_only().timeout(REQUEST_TIMEOUT).build()?;

        Ok(Self { http, crumb: RwLock::new(None) })
    }

    /// Fetches quotes for `symbols`, retrying once with a fresh crumb when
    /// Yahoo rejects the cached one.
    pub async fn quote(&self, symbols: &[String], lang: &str, region: &str) -> Result<Vec<YahooQuote>, YahooError> {
        let joined = symbols.join(",");

        match self.quote_once(&joined, lang, region, false).await {
            Err(error) if is_stale_credentials(&error) => self.quote_once(&joined, lang, region, true).await,
            result => result,
        }
    }

    async fn quote_once(&self, symbols: &str, lang: &str, region: &str, refresh_crumb: bool) -> Result<Vec<YahooQuote>, YahooError> {
        let crumb = self.crumb(refresh_crumb).await?;
        let response = self
            .http
            .get(QUOTE_URL)
            .query(&[("symbols", symbols), ("lang", lang), ("region", region), ("crumb", &crumb)])
            .header(ACCEPT, "application/json")
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
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

    async fn crumb(&self, refresh: bool) -> Result<String, YahooError> {
        if !refresh {
            // Read the cached crumb into an owned value so the read guard is
            // released before we ever ask for the write guard.
            let cached = self.crumb.read().await.clone();
            if let Some(crumb) = cached {
                return Ok(crumb);
            }
        }

        let mut cached = self.crumb.write().await;
        if !refresh && let Some(crumb) = cached.clone() {
            return Ok(crumb);
        }

        // This request exists purely for its `set-cookie` headers, which the
        // client's cookie store keeps for the crumb and quote calls.
        self.http.get(COOKIE_URL).header(ACCEPT, "text/html,application/xhtml+xml,application/xml").send().await?;

        let response = self
            .http
            .get(CRUMB_URL)
            .header(ACCEPT, "*/*")
            .header(ORIGIN, "https://finance.yahoo.com")
            .header(REFERER, COOKIE_URL)
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

        *cached = Some(crumb.clone());

        Ok(crumb)
    }
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
