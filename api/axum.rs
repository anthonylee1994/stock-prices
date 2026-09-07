use std::sync::Arc;

use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

use stock_prices::{StockPricesService, create_app};

/// Vercel Function entrypoint.
///
/// `vercel.json` rewrites every path here, so the same `create_app` router that
/// `src/main.rs` serves locally also handles production traffic. `VercelLayer`
/// translates Vercel's function bridge protocol into `axum` requests, which
/// keeps every controller, error mapping and CORS setting unchanged.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let router = create_app(Arc::new(StockPricesService::new()?));
    let app = ServiceBuilder::new().layer(VercelLayer::new()).service(router);

    vercel_runtime::run(app).await
}
