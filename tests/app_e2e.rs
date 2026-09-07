use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use stock_prices::error::ApiError;
use stock_prices::{Quote, QuoteProvider, create_app};
use tower::ServiceExt;

/// Records the symbols it was asked for so tests can assert on parsing, the way
/// the Jest suite asserted on `getQuotes` call arguments.
#[derive(Default)]
struct StubQuoteProvider {
    calls: Mutex<Vec<Vec<String>>>,
}

#[async_trait]
impl QuoteProvider for StubQuoteProvider {
    async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<Quote>, ApiError> {
        self.calls.lock().expect("calls lock").push(symbols.to_vec());

        Ok(vec![Quote {
            symbol: "AAPL".to_owned(),
            name: Some("Apple Inc.".to_owned()),
            market: Some("us_market".to_owned()),
            current_price: Some(230.12),
            ..Quote::default()
        }])
    }
}

/// Always fails, standing in for an unreachable Yahoo Finance.
struct FailingQuoteProvider;

#[async_trait]
impl QuoteProvider for FailingQuoteProvider {
    async fn get_quotes(&self, _symbols: &[String]) -> Result<Vec<Quote>, ApiError> {
        Err(ApiError::bad_gateway("Failed to fetch quotes"))
    }
}

fn create_test_app() -> (Router, Arc<StubQuoteProvider>) {
    let stock_prices_service = Arc::new(StubQuoteProvider::default());

    (create_app(stock_prices_service.clone()), stock_prices_service)
}

async fn send(app: Router, method: &str, uri: &str) -> (StatusCode, Option<String>, Value) {
    let request = Request::builder().method(method).uri(uri).body(Body::empty()).expect("request builds");
    let response = app.oneshot(request).await.expect("response");
    let status = response.status();
    let content_type = response.headers().get("content-type").and_then(|value| value.to_str().ok()).map(str::to_owned);
    let body = response.into_body().collect().await.expect("body").to_bytes();
    let body = if body.is_empty() { Value::Null } else { serde_json::from_slice(&body).expect("body is json") };

    (status, content_type, body)
}

#[tokio::test]
async fn returns_api_metadata() {
    let (app, _) = create_test_app();

    let (status, content_type, body) = send(app, "GET", "/").await;

    assert_eq!(status, StatusCode::OK);
    assert!(content_type.expect("content-type").contains("json"));
    assert_eq!(body, json!({"message": "Stock Prices API", "version": "1.0.0"}));
}

#[tokio::test]
async fn returns_quotes() {
    let (app, stock_prices_service) = create_test_app();

    let (status, content_type, body) = send(app, "GET", "/quotes?symbols=%20AAPL,%20MSFT%20,,%200700.HK%20").await;

    assert_eq!(status, StatusCode::OK);
    assert!(content_type.expect("content-type").contains("json"));
    assert_eq!(body, json!({"quotes": [{"symbol": "AAPL", "name": "Apple Inc.", "market": "us_market", "currentPrice": 230.12}]}),);
    assert_eq!(
        *stock_prices_service.calls.lock().expect("calls lock"),
        vec![vec!["AAPL".to_owned(), "MSFT".to_owned(), "0700.HK".to_owned()]],
    );
}

#[tokio::test]
async fn rejects_missing_symbols() {
    let (app, stock_prices_service) = create_test_app();

    let (status, _, body) = send(app, "GET", "/quotes").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, json!({"message": "symbols is required", "error": "Bad Request", "statusCode": 400}));
    assert!(stock_prices_service.calls.lock().expect("calls lock").is_empty());
}

#[tokio::test]
async fn rejects_blank_symbols() {
    let (app, stock_prices_service) = create_test_app();

    let (status, _, _) = send(app, "GET", "/quotes?symbols=%20,%20,%20").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(stock_prices_service.calls.lock().expect("calls lock").is_empty());
}

#[tokio::test]
async fn rejects_unsupported_quote_methods() {
    let (app, _) = create_test_app();

    let (status, _, body) = send(app, "POST", "/quotes").await;

    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(body, json!({"message": "Method Not Allowed", "statusCode": 405}));
}

#[tokio::test]
async fn rejects_unsupported_root_methods() {
    let (app, _) = create_test_app();

    let (status, _, _) = send(app, "POST", "/").await;

    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn reports_upstream_failures_as_bad_gateway() {
    let app = create_app(Arc::new(FailingQuoteProvider));

    let (status, _, body) = send(app, "GET", "/quotes?symbols=AAPL").await;

    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body, json!({"message": "Failed to fetch quotes", "error": "Bad Gateway", "statusCode": 502}));
}

#[tokio::test]
async fn allows_cross_origin_reads() {
    let (app, _) = create_test_app();

    let request = Request::builder()
        .method("GET")
        .uri("/")
        .header("origin", "https://example.com")
        .body(Body::empty())
        .expect("request builds");
    let response = app.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("access-control-allow-origin").expect("cors header"), "*");
}
