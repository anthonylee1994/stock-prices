use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

use crate::app_controller;
use crate::stock_prices::{self, QuoteProvider};

/// Mounts every controller and applies the shared CORS policy.
pub fn create_app(stock_prices_service: Arc<dyn QuoteProvider>) -> Router {
    Router::new().merge(app_controller::routes()).merge(stock_prices::routes(stock_prices_service)).layer(cors())
}

fn cors() -> CorsLayer {
    CorsLayer::new().allow_origin(Any).allow_methods([Method::GET, Method::OPTIONS]).allow_headers(Any)
}
