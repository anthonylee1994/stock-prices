use std::error::Error;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::ApiError;
use crate::stock_prices::types::Quote;
use crate::stock_prices::yahoo::{YahooClient, YahooError, YahooQuote};

const LANG: &str = "zh-HK";
const REGION: &str = "HK";

/// Quote source used by the controller.
///
/// The controller depends on this trait rather than [`StockPricesService`] so
/// tests can swap in a stub, the way the Nest module overrode its provider.
#[async_trait]
pub trait QuoteProvider: Send + Sync {
    async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<Quote>, ApiError>;
}

pub struct StockPricesService {
    yahoo: YahooClient,
}

impl StockPricesService {
    pub fn new() -> Result<Self, YahooError> {
        Ok(Self { yahoo: YahooClient::new()? })
    }
}

#[async_trait]
impl QuoteProvider for StockPricesService {
    async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<Quote>, ApiError> {
        match self.yahoo.quote(symbols, LANG, REGION).await {
            Ok(quotes) => Ok(quotes.into_iter().map(map_quote).collect()),
            Err(error) => {
                // Nest attached the upstream error as `cause`; we log the whole
                // chain here so the detail is not lost while the client still
                // only sees a 502.
                eprintln!("Failed to fetch quotes: {}", error_chain(&error));
                Err(ApiError::bad_gateway("Failed to fetch quotes"))
            }
        }
    }
}

fn map_quote(quote: YahooQuote) -> Quote {
    Quote {
        symbol: quote.symbol,
        name: quote.long_name,
        market: quote.market,
        current_price: quote.regular_market_price,
        change: quote.regular_market_change,
        percent_change: quote.regular_market_change_percent,
        high_price: quote.regular_market_day_high,
        low_price: quote.regular_market_day_low,
        open_price: quote.regular_market_open,
        regular_market_time: timestamp(quote.regular_market_time),
        previous_close_price: quote.regular_market_previous_close,
        pre_market_price: quote.pre_market_price,
        pre_market_change: quote.pre_market_change,
        pre_market_time: timestamp(quote.pre_market_time),
        pre_market_change_percent: quote.pre_market_change_percent,
        post_market_price: quote.post_market_price,
        post_market_change: quote.post_market_change,
        post_market_change_percent: quote.post_market_change_percent,
        post_market_time: timestamp(quote.post_market_time),
        forward_pe: quote.forward_pe,
        price_to_book: quote.price_to_book,
        dividend_yield: quote.dividend_yield,
    }
}

fn timestamp(seconds: Option<i64>) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(seconds?, 0)
}

/// Flattens an error and its causes into `outer: inner: root`.
fn error_chain(error: &dyn Error) -> String {
    let mut parts = vec![error.to_string()];
    let mut source = error.source();

    while let Some(current) = source {
        parts.push(current.to_string());
        source = current.source();
    }

    parts.join(": ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_yahoo_finance_quotes_to_api_quotes() {
        let raw = YahooQuote {
            symbol: "AAPL".to_owned(),
            quote_type: Some("EQUITY".to_owned()),
            long_name: Some("Apple Inc.".to_owned()),
            market: Some("us_market".to_owned()),
            regular_market_price: Some(230.12),
            regular_market_change: Some(1.23),
            regular_market_change_percent: Some(0.54),
            regular_market_day_high: Some(231.0),
            regular_market_day_low: Some(228.4),
            regular_market_open: Some(229.0),
            regular_market_time: Some(1_777_455_000),
            regular_market_previous_close: Some(228.89),
            pre_market_price: Some(229.5),
            pre_market_change: Some(0.61),
            pre_market_change_percent: Some(0.27),
            pre_market_time: Some(1_777_449_600),
            post_market_price: Some(230.4),
            post_market_change: Some(0.28),
            post_market_change_percent: Some(0.12),
            post_market_time: Some(1_777_492_800),
            forward_pe: Some(28.5),
            price_to_book: Some(45.3),
            dividend_yield: Some(0.005),
        };

        let quote = map_quote(raw);

        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.name.as_deref(), Some("Apple Inc."));
        assert_eq!(quote.market.as_deref(), Some("us_market"));
        assert_eq!(quote.current_price, Some(230.12));
        assert_eq!(quote.change, Some(1.23));
        assert_eq!(quote.percent_change, Some(0.54));
        assert_eq!(quote.high_price, Some(231.0));
        assert_eq!(quote.low_price, Some(228.4));
        assert_eq!(quote.open_price, Some(229.0));
        assert_eq!(quote.previous_close_price, Some(228.89));
        assert_eq!(quote.pre_market_price, Some(229.5));
        assert_eq!(quote.post_market_price, Some(230.4));
        assert_eq!(quote.forward_pe, Some(28.5));
        assert_eq!(quote.price_to_book, Some(45.3));
        assert_eq!(quote.dividend_yield, Some(0.005));
        let serialized = serde_json::to_value(&quote).expect("quote serializes");
        assert_eq!(serialized["regularMarketTime"], serde_json::json!("2026-04-29T09:30:00.000Z"));
        assert_eq!(serialized["preMarketTime"], serde_json::json!("2026-04-29T08:00:00.000Z"));
        assert_eq!(serialized["postMarketTime"], serde_json::json!("2026-04-29T20:00:00.000Z"));
    }

    #[test]
    fn omits_absent_fields() {
        let raw = YahooQuote {
            symbol: "AAPL".to_owned(),
            regular_market_price: Some(230.12),
            ..YahooQuote::default()
        };

        let quote = serde_json::to_value(map_quote(raw)).expect("quote serializes");

        assert_eq!(quote, serde_json::json!({"symbol": "AAPL", "currentPrice": 230.12}));
    }
}
