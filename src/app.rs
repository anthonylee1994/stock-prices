use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tower_http::cors::CorsLayer;

use crate::handlers::{get_quotes, get_root};
use crate::stock_prices::StockPricesService;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) stock_prices: StockPricesService,
}

pub(crate) fn router() -> Router {
    let app_state = AppState {
        stock_prices: StockPricesService::new(),
    };

    Router::new()
        .route("/", get(get_root))
        .route("/quotes", get(get_quotes))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(app_state))
}
