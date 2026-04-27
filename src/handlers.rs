use std::sync::Arc;

use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::error::ApiError;
use crate::response::{ToonResponse, toon_response};
use crate::stock_prices::Quote;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) async fn get_root() -> Result<ToonResponse, ApiError> {
    toon_response(&RootResponse {
        message: "Stock Prices API",
        version: VERSION,
    })
}

#[derive(Deserialize)]
pub(crate) struct QuotesQuery {
    symbols: Option<String>,
}

pub(crate) async fn get_quotes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<QuotesQuery>,
) -> Result<ToonResponse, ApiError> {
    let symbols = query.symbols.ok_or(ApiError::BadRequest {
        error: "symbols is required",
    })?;
    let symbols = symbols
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if symbols.is_empty() {
        return Err(ApiError::BadRequest {
            error: "symbols is required",
        });
    }

    let quotes = state.stock_prices.get_quotes(&symbols).await?;

    toon_response(&QuotesResponse { quotes })
}

#[derive(Serialize)]
struct RootResponse<'a> {
    message: &'a str,
    version: &'a str,
}

#[derive(Serialize)]
struct QuotesResponse {
    quotes: Vec<Quote>,
}

#[cfg(test)]
mod tests {
    use toon_format::encode_default;

    use super::*;

    #[test]
    fn encodes_root_object_as_toon() {
        let output = encode_default(&RootResponse {
            message: "Stock Prices API",
            version: "1.0.0",
        })
        .unwrap();

        assert_eq!(output, "message: Stock Prices API\nversion: \"1.0.0\"");
    }

    #[test]
    fn encodes_quotes_as_toon_table() {
        let output = encode_default(&QuotesResponse {
            quotes: vec![Quote {
                symbol: "NVDA".to_owned(),
                name: Some("NVIDIA Corporation".to_owned()),
                market: Some("us_market".to_owned()),
                current_price: Some(188.54),
                change: Some(-1.5),
                percent_change: Some(0.1),
                high_price: Some(190.0),
                low_price: Some(180.0),
                open_price: Some(181.0),
                regular_market_time: Some("2026-02-11T13:49:16.000Z".to_owned()),
                previous_close_price: Some(190.04),
                pre_market_price: Some(191.8799),
                pre_market_change: Some(3.3399048),
                pre_market_time: Some("2026-02-11T13:49:16.000Z".to_owned()),
                pre_market_change_percent: Some(1.771457),
                post_market_price: Some(188.0),
                post_market_change: Some(-0.54),
                post_market_change_percent: Some(-0.286),
                post_market_time: Some("2026-02-11T21:00:00.000Z".to_owned()),
                forward_pe: Some(18.529821),
                price_to_book: Some(32.178616),
                dividend_yield: Some(0.02),
            }],
        })
        .unwrap();

        assert!(output.starts_with("quotes[1]{symbol,name,market,currentPrice"));
        assert!(output.contains("\n  NVDA,NVIDIA Corporation,us_market,188.54,-1.5,"));
        assert!(output.contains("forwardPE"));
    }
}
