use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::error::method_not_allowed;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize)]
pub struct ApiMetadata {
    message: &'static str,
    version: &'static str,
}

pub fn routes() -> Router {
    Router::new().route("/", get(index).fallback(method_not_allowed))
}

async fn index() -> Json<ApiMetadata> {
    Json(ApiMetadata {
        message: "Stock Prices API",
        version: VERSION,
    })
}
