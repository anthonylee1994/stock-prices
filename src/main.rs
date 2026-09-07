use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use stock_prices::{StockPricesService, create_app};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = create_app(Arc::new(StockPricesService::new()?));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_owned());
    let address: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!("Server is running on http://localhost:{port}");
    axum::serve(listener, app).with_graceful_shutdown(shutdown()).await?;

    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
