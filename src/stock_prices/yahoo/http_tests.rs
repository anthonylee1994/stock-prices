use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use axum::extract::{Query, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use tokio::sync::Barrier;
use tokio::task::{JoinHandle, JoinSet};

use super::*;

struct MockState {
    sessions: AtomicUsize,
    quotes: AtomicUsize,
    expired: AtomicBool,
    reject_all: AtomicBool,
    fail_crumb: AtomicBool,
    rejection_status: StatusCode,
    rejection_description: &'static str,
    stale_requests: Barrier,
}

impl MockState {
    fn new(status: StatusCode, concurrent: usize) -> Self {
        Self {
            sessions: AtomicUsize::new(0),
            quotes: AtomicUsize::new(0),
            expired: AtomicBool::new(false),
            reject_all: AtomicBool::new(false),
            fail_crumb: AtomicBool::new(false),
            rejection_status: status,
            rejection_description: "Forbidden",
            stale_requests: Barrier::new(concurrent),
        }
    }
}

struct MockYahoo {
    state: Arc<MockState>,
    client: Arc<YahooClient>,
    server: JoinHandle<()>,
}

impl MockYahoo {
    async fn start(state: MockState) -> Self {
        let state = Arc::new(state);
        let app = Router::new()
            .route("/cookie", get(cookie))
            .route("/crumb", get(crumb))
            .route("/quotes", get(quotes))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind mock");
        let base = format!("http://{}", listener.local_addr().expect("mock address"));
        let server = tokio::spawn(async move { axum::serve(listener, app).await.expect("mock server") });
        let mut client = YahooClient::new().expect("client");
        client.endpoints = Endpoints {
            cookie: format!("{base}/cookie"),
            crumb: format!("{base}/crumb"),
            quote: format!("{base}/quotes"),
        };

        Self {
            state,
            client: Arc::new(client),
            server,
        }
    }
}

impl Drop for MockYahoo {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn cookie(State(state): State<Arc<MockState>>) -> impl IntoResponse {
    let generation = state.sessions.fetch_add(1, Ordering::SeqCst) + 1;
    ([(header::SET_COOKIE, format!("session={generation}; Path=/"))], "")
}

async fn crumb(State(state): State<Arc<MockState>>, headers: HeaderMap) -> Response {
    if state.fail_crumb.load(Ordering::SeqCst) {
        return (StatusCode::SERVICE_UNAVAILABLE, "not available").into_response();
    }
    let session = headers.get(header::COOKIE).expect("session cookie").to_str().expect("cookie text");
    session.replace("session=", "crumb-").into_response()
}

async fn quotes(State(state): State<Arc<MockState>>, headers: HeaderMap, Query(query): Query<HashMap<String, String>>) -> Response {
    state.quotes.fetch_add(1, Ordering::SeqCst);
    let crumb = query.get("crumb").expect("crumb query");
    let cookie = headers.get(header::COOKIE).expect("session cookie").to_str().expect("cookie text");
    if cookie.replace("session=", "crumb-") != *crumb {
        return (StatusCode::BAD_REQUEST, "cookie/crumb mismatch").into_response();
    }
    assert_eq!(query.get("symbols").map(String::as_str), Some("AAPL"));
    assert_eq!(query.get("lang").map(String::as_str), Some("zh-HK"));
    assert_eq!(query.get("region").map(String::as_str), Some("HK"));

    let stale = crumb == "crumb-1" && state.expired.load(Ordering::SeqCst);
    if stale {
        state.stale_requests.wait().await;
    }
    if stale || state.reject_all.load(Ordering::SeqCst) {
        return (
            state.rejection_status,
            Json(serde_json::json!({"finance": {"error": {"code": "Denied", "description": state.rejection_description}}})),
        )
            .into_response();
    }
    Json(serde_json::json!({"quoteResponse": {"result": [{"symbol": "AAPL", "regularMarketPrice": 230.12}, {"symbol": "DEAD", "quoteType": "NONE"}], "error": null}})).into_response()
}

async fn fetch(client: &YahooClient) -> Result<Vec<YahooQuote>, YahooError> {
    tokio::time::timeout(Duration::from_secs(5), client.quote(&["AAPL".to_owned()], "zh-HK", "HK"))
        .await
        .expect("quote deadline")
}

#[tokio::test]
async fn refreshes_json_auth_statuses_without_matching_description() {
    for status in [StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN] {
        let mock = MockYahoo::start(MockState::new(status, 1)).await;
        mock.state.expired.store(true, Ordering::SeqCst);
        let quotes = fetch(&mock.client).await.expect("refresh succeeds");
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].symbol, "AAPL");
        assert_eq!(quotes[0].regular_market_price, Some(230.12));
        assert_eq!(mock.state.sessions.load(Ordering::SeqCst), 2);
        assert_eq!(mock.state.quotes.load(Ordering::SeqCst), 2);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_expired_requests_share_one_refresh() {
    let mut state = MockState::new(StatusCode::UNAUTHORIZED, 8);
    state.rejection_description = "Invalid Crumb";
    let mock = MockYahoo::start(state).await;
    fetch(&mock.client).await.expect("warm session");
    mock.state.expired.store(true, Ordering::SeqCst);
    let mut tasks = JoinSet::new();
    for _ in 0..8 {
        let client = mock.client.clone();
        tasks.spawn(async move { fetch(&client).await.expect("refreshed quotes") });
    }
    while let Some(result) = tasks.join_next().await {
        assert_eq!(result.expect("request task")[0].symbol, "AAPL");
    }
    assert_eq!(mock.state.sessions.load(Ordering::SeqCst), 2);
    assert_eq!(mock.state.quotes.load(Ordering::SeqCst), 17);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_cold_requests_share_initialization() {
    let mock = MockYahoo::start(MockState::new(StatusCode::UNAUTHORIZED, 1)).await;
    let mut tasks = JoinSet::new();
    for _ in 0..8 {
        let client = mock.client.clone();
        tasks.spawn(async move { fetch(&client).await.expect("quotes") });
    }
    while let Some(result) = tasks.join_next().await {
        assert_eq!(result.expect("request task")[0].symbol, "AAPL");
    }
    assert_eq!(mock.state.sessions.load(Ordering::SeqCst), 1);
    assert_eq!(mock.state.quotes.load(Ordering::SeqCst), 8);
}

#[tokio::test]
async fn stops_after_one_retry_and_does_not_retry_other_statuses() {
    for (status, attempts) in [(StatusCode::FORBIDDEN, 2), (StatusCode::TOO_MANY_REQUESTS, 1), (StatusCode::INTERNAL_SERVER_ERROR, 1)] {
        let mock = MockYahoo::start(MockState::new(status, 1)).await;
        mock.state.reject_all.store(true, Ordering::SeqCst);
        assert!(fetch(&mock.client).await.is_err());
        assert_eq!(mock.state.sessions.load(Ordering::SeqCst), attempts);
        assert_eq!(mock.state.quotes.load(Ordering::SeqCst), attempts);
    }
}

#[tokio::test]
async fn recovers_after_failed_session_initialization() {
    let mock = MockYahoo::start(MockState::new(StatusCode::UNAUTHORIZED, 1)).await;
    mock.state.fail_crumb.store(true, Ordering::SeqCst);
    assert!(fetch(&mock.client).await.is_err());
    assert_eq!(mock.state.quotes.load(Ordering::SeqCst), 0);
    mock.state.fail_crumb.store(false, Ordering::SeqCst);
    assert_eq!(fetch(&mock.client).await.expect("recovered")[0].symbol, "AAPL");
    assert_eq!(mock.state.sessions.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn refreshing_does_not_change_an_in_flight_sessions_cookies() {
    let mock = MockYahoo::start(MockState::new(StatusCode::UNAUTHORIZED, 1)).await;
    let old = mock.client.session(None).await.expect("original session");
    let fresh = mock.client.session(Some(&old)).await.expect("new session");

    // Simulate a request paused after taking its session snapshot: it must
    // still send the original cookie even after a different request refreshes.
    for session in [&old, &fresh] {
        let quotes = mock.client.quote_once("AAPL", "zh-HK", "HK", session).await.expect("matching cookie and crumb");
        assert_eq!(quotes[0].symbol, "AAPL");
    }
    assert_eq!(mock.state.sessions.load(Ordering::SeqCst), 2);
}
