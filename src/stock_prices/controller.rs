use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, method_not_allowed};
use crate::stock_prices::service::QuoteProvider;
use crate::stock_prices::types::Quote;

pub fn routes(stock_prices_service: Arc<dyn QuoteProvider>) -> Router {
    Router::new().route("/quotes", get(index).fallback(method_not_allowed)).with_state(stock_prices_service)
}

#[derive(Debug, Deserialize)]
struct QuotesQuery {
    symbols: Option<String>,
}

#[derive(Debug, Serialize)]
struct QuotesResponse {
    quotes: Vec<Quote>,
}

async fn index(State(stock_prices_service): State<Arc<dyn QuoteProvider>>, Query(query): Query<QuotesQuery>) -> Result<Json<QuotesResponse>, ApiError> {
    let symbols = parse_symbols(query.symbols.as_deref()).ok_or_else(|| ApiError::bad_request("symbols is required"))?;
    let quotes = stock_prices_service.get_quotes(&symbols).await?;

    Ok(Json(QuotesResponse { quotes }))
}

/// Splits the comma-separated `symbols` query parameter.
///
/// Non-Hong Kong symbols use `-` where Yahoo's class separator would be a `.`
/// (`BRK.B` becomes `BRK-B`), while `.HK` tickers keep their suffix. Blank
/// entries are dropped, and an empty result is treated as no input at all.
fn parse_symbols(input: Option<&str>) -> Option<Vec<String>> {
    let input = input?;
    if input.is_empty() {
        return None;
    }

    let symbols: Vec<String> = input
        .split(',')
        .map(|symbol| {
            if symbol.contains('.') && !symbol.contains(".HK") {
                symbol.replacen('.', "-", 1).trim().to_owned()
            } else {
                symbol.trim().to_owned()
            }
        })
        .filter(|symbol| !symbol.is_empty())
        .collect();

    if symbols.is_empty() { None } else { Some(symbols) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_separated_symbols() {
        assert_eq!(parse_symbols(Some(" AAPL, MSFT ,, 0700.HK ")), Some(vec!["AAPL".to_owned(), "MSFT".to_owned(), "0700.HK".to_owned()]),);
    }

    #[test]
    fn rewrites_non_hong_kong_class_separators() {
        assert_eq!(parse_symbols(Some("BRK.B")), Some(vec!["BRK-B".to_owned()]));
        assert_eq!(parse_symbols(Some("0700.HK")), Some(vec!["0700.HK".to_owned()]));
    }

    #[test]
    fn rejects_missing_symbols() {
        assert_eq!(parse_symbols(None), None);
        assert_eq!(parse_symbols(Some("")), None);
    }

    #[test]
    fn rejects_blank_symbol_lists() {
        assert_eq!(parse_symbols(Some(" , , ")), None);
    }
}
