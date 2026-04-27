use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Clone)]
pub(crate) struct StockPricesService {
    client: Client,
    crumb: Arc<Mutex<Option<String>>>,
}

impl StockPricesService {
    pub(crate) fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
                    AppleWebKit/537.36 (KHTML, like Gecko) \
                    Chrome/124.0.0.0 Safari/537.36",
                )
                .cookie_store(true)
                .build()
                .expect("reqwest client config should be valid"),
            crumb: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<Quote>, ApiError> {
        let mut response = self.send_quotes_request(symbols, false).await?;

        if matches!(
            response.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            self.clear_crumb();
            response = self.send_quotes_request(symbols, true).await?;
        }

        if !response.status().is_success() {
            return Err(ApiError::UpstreamStatus(response.status()));
        }

        let response = response
            .json::<YahooQuoteResponse>()
            .await
            .map_err(ApiError::YahooFinance)?;

        Ok(response
            .quote_response
            .result
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    async fn send_quotes_request(
        &self,
        symbols: &[String],
        force_crumb_refresh: bool,
    ) -> Result<reqwest::Response, ApiError> {
        let crumb = self.get_crumb(force_crumb_refresh).await?;

        let response = self
            .client
            .get("https://query1.finance.yahoo.com/v7/finance/quote")
            .query(&[
                ("symbols", symbols.join(",")),
                ("lang", "zh-HK".to_owned()),
                ("region", "HK".to_owned()),
                ("crumb", crumb),
            ])
            .send()
            .await
            .map_err(ApiError::YahooFinance)?;

        Ok(response)
    }

    async fn get_crumb(&self, force_refresh: bool) -> Result<String, ApiError> {
        if !force_refresh
            && let Some(crumb) = self
                .crumb
                .lock()
                .expect("crumb mutex should not be poisoned")
                .clone()
        {
            return Ok(crumb);
        }

        self.client
            .get("https://fc.yahoo.com")
            .send()
            .await
            .map_err(ApiError::YahooFinance)?;

        let crumb = self
            .client
            .get("https://query1.finance.yahoo.com/v1/test/getcrumb")
            .send()
            .await
            .map_err(ApiError::YahooFinance)?
            .error_for_status()
            .map_err(ApiError::YahooFinance)?
            .text()
            .await
            .map_err(ApiError::YahooFinance)?;

        let crumb = crumb.trim().to_owned();

        if crumb.is_empty() || crumb.contains("Too Many Requests") {
            return Err(ApiError::YahooCrumb);
        }

        *self
            .crumb
            .lock()
            .expect("crumb mutex should not be poisoned") = Some(crumb.clone());

        Ok(crumb)
    }

    fn clear_crumb(&self) {
        *self
            .crumb
            .lock()
            .expect("crumb mutex should not be poisoned") = None;
    }
}

#[derive(Deserialize)]
struct YahooQuoteResponse {
    #[serde(rename = "quoteResponse")]
    quote_response: YahooQuoteResult,
}

#[derive(Deserialize)]
struct YahooQuoteResult {
    result: Vec<YahooQuote>,
}

#[derive(Deserialize)]
struct YahooQuote {
    symbol: String,
    #[serde(rename = "longName")]
    long_name: Option<String>,
    market: Option<String>,
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(rename = "regularMarketChange")]
    regular_market_change: Option<f64>,
    #[serde(rename = "regularMarketChangePercent")]
    regular_market_change_percent: Option<f64>,
    #[serde(rename = "regularMarketDayHigh")]
    regular_market_day_high: Option<f64>,
    #[serde(rename = "regularMarketDayLow")]
    regular_market_day_low: Option<f64>,
    #[serde(rename = "regularMarketOpen")]
    regular_market_open: Option<f64>,
    #[serde(rename = "regularMarketTime")]
    regular_market_time: Option<i64>,
    #[serde(rename = "regularMarketPreviousClose")]
    regular_market_previous_close: Option<f64>,
    #[serde(rename = "preMarketPrice")]
    pre_market_price: Option<f64>,
    #[serde(rename = "preMarketChange")]
    pre_market_change: Option<f64>,
    #[serde(rename = "preMarketTime")]
    pre_market_time: Option<i64>,
    #[serde(rename = "preMarketChangePercent")]
    pre_market_change_percent: Option<f64>,
    #[serde(rename = "postMarketPrice")]
    post_market_price: Option<f64>,
    #[serde(rename = "postMarketChange")]
    post_market_change: Option<f64>,
    #[serde(rename = "postMarketChangePercent")]
    post_market_change_percent: Option<f64>,
    #[serde(rename = "postMarketTime")]
    post_market_time: Option<i64>,
    #[serde(rename = "forwardPE")]
    forward_pe: Option<f64>,
    #[serde(rename = "priceToBook")]
    price_to_book: Option<f64>,
    #[serde(rename = "dividendYield")]
    dividend_yield: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Quote {
    pub(crate) symbol: String,
    pub(crate) name: Option<String>,
    pub(crate) market: Option<String>,
    pub(crate) current_price: Option<f64>,
    pub(crate) change: Option<f64>,
    pub(crate) percent_change: Option<f64>,
    pub(crate) high_price: Option<f64>,
    pub(crate) low_price: Option<f64>,
    pub(crate) open_price: Option<f64>,
    pub(crate) regular_market_time: Option<String>,
    pub(crate) previous_close_price: Option<f64>,
    pub(crate) pre_market_price: Option<f64>,
    pub(crate) pre_market_change: Option<f64>,
    pub(crate) pre_market_time: Option<String>,
    pub(crate) pre_market_change_percent: Option<f64>,
    pub(crate) post_market_price: Option<f64>,
    pub(crate) post_market_change: Option<f64>,
    pub(crate) post_market_change_percent: Option<f64>,
    pub(crate) post_market_time: Option<String>,
    #[serde(rename = "forwardPE")]
    pub(crate) forward_pe: Option<f64>,
    pub(crate) price_to_book: Option<f64>,
    pub(crate) dividend_yield: Option<f64>,
}

impl From<YahooQuote> for Quote {
    fn from(quote: YahooQuote) -> Self {
        Self {
            symbol: quote.symbol,
            name: quote.long_name,
            market: quote.market,
            current_price: quote.regular_market_price,
            change: quote.regular_market_change,
            percent_change: quote.regular_market_change_percent,
            high_price: quote.regular_market_day_high,
            low_price: quote.regular_market_day_low,
            open_price: quote.regular_market_open,
            regular_market_time: quote.regular_market_time.and_then(epoch_to_iso),
            previous_close_price: quote.regular_market_previous_close,
            pre_market_price: quote.pre_market_price,
            pre_market_change: quote.pre_market_change,
            pre_market_time: quote.pre_market_time.and_then(epoch_to_iso),
            pre_market_change_percent: quote.pre_market_change_percent,
            post_market_price: quote.post_market_price,
            post_market_change: quote.post_market_change,
            post_market_change_percent: quote.post_market_change_percent,
            post_market_time: quote.post_market_time.and_then(epoch_to_iso),
            forward_pe: quote.forward_pe,
            price_to_book: quote.price_to_book,
            dividend_yield: quote.dividend_yield,
        }
    }
}

fn epoch_to_iso(epoch_seconds: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(epoch_seconds, 0)
        .map(|time| time.to_rfc3339_opts(SecondsFormat::Millis, true))
}
