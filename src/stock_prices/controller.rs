use std::sync::Arc;

use axum::extract::rejection::QueryRejection;
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

async fn index(State(stock_prices_service): State<Arc<dyn QuoteProvider>>, query: Result<Query<QuotesQuery>, QueryRejection>) -> Result<Json<QuotesResponse>, ApiError> {
    let Query(query) = query.map_err(|_| ApiError::bad_request("Invalid query parameters"))?;
    let symbols = parse_symbols(query.symbols.as_deref()).ok_or_else(|| ApiError::bad_request("symbols is required"))?;
    let quotes = stock_prices_service.get_quotes(&symbols).await?;

    Ok(Json(QuotesResponse { quotes }))
}

/// Splits the comma-separated `symbols` query parameter.
///
/// Recognized legacy class aliases use Yahoo's `-` separator. Other dotted
/// symbols keep their exchange suffix. Blank entries are dropped.
fn parse_symbols(input: Option<&str>) -> Option<Vec<String>> {
    let input = input?;
    if input.is_empty() {
        return None;
    }

    let symbols: Vec<String> = input
        .split(',')
        .map(str::trim)
        .map(|symbol| {
            match symbol {
                "BRK.A" => "BRK-A",
                "BRK.B" => "BRK-B",
                "BF.A" => "BF-A",
                "BF.B" => "BF-B",
                _ => symbol,
            }
            .to_owned()
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

    #[test]
    fn normalization_preserves_symbols_and_is_idempotent() {
        let cases = [
            ("BRK.A", "BRK-A"),
            ("BRK.B", "BRK-B"),
            ("BF.A", "BF-A"),
            ("BF.B", "BF-B"),
            ("7203.T", "7203.T"),
            ("VOD.L", "VOD.L"),
            ("0700.hk", "0700.hk"),
            ("ABC.A", "ABC.A"),
            ("X.HK.B", "X.HK.B"),
            ("^GSPC", "^GSPC"),
            ("BTC-USD", "BTC-USD"),
            ("AAPL", "AAPL"),
        ];
        for (left, normalized_left) in cases {
            for (right, normalized_right) in cases {
                for whitespace in ["", " ", "\t", "\n", "\u{2003}"] {
                    let input = format!(",{whitespace}{left}{whitespace},,{right},{left},");
                    let normalized = parse_symbols(Some(&input)).expect("symbols");
                    assert_eq!(normalized, vec![normalized_left, normalized_right, normalized_left]);
                    assert_eq!(parse_symbols(Some(&normalized.join(","))), Some(normalized));
                }
            }
        }
    }
}
