mod app;
mod error;
mod handlers;
mod response;
mod stock_prices;

use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_owned());
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;

    let app = app::router();

    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Server is running on http://localhost:{port}");
    println!("Stock Prices API is ready");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
