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
    #[cfg(unix)]
    {
        let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }

    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(all(test, unix))]
mod tests {
    use async_trait::async_trait;
    use stock_prices::error::ApiError;
    use stock_prices::{Quote, QuoteProvider};
    use tokio::sync::Notify;
    use tokio::time::{Duration, timeout};

    use super::*;

    #[derive(Default)]
    struct SlowProvider {
        started: Notify,
        finish: Notify,
    }

    #[async_trait]
    impl QuoteProvider for SlowProvider {
        async fn get_quotes(&self, _symbols: &[String]) -> Result<Vec<Quote>, ApiError> {
            self.started.notify_one();
            self.finish.notified().await;
            Ok(vec![Quote {
                symbol: "AAPL".to_owned(),
                ..Quote::default()
            }])
        }
    }

    #[tokio::test]
    async fn sigterm_drains_an_in_flight_quote_request() {
        timeout(Duration::from_secs(5), async {
            let provider = Arc::new(SlowProvider::default());
            let app = create_app(provider.clone());
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let url = format!("http://{}/quotes?symbols=AAPL", listener.local_addr().expect("address"));
            let observed = Arc::new(Notify::new());
            let shutdown_observed = observed.clone();
            let server = tokio::spawn(async move {
                axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        shutdown().await;
                        shutdown_observed.notify_one();
                    })
                    .await
                    .expect("serve");
            });
            let request = tokio::spawn(async move {
                reqwest::Client::new()
                    .get(url)
                    .send()
                    .await
                    .expect("response")
                    .error_for_status()
                    .expect("success")
                    .json::<serde_json::Value>()
                    .await
                    .expect("JSON")
            });
            provider.started.notified().await;
            assert!(std::process::Command::new("kill").args(["-TERM", &std::process::id().to_string()]).status().expect("signal").success());
            observed.notified().await;
            assert!(!server.is_finished(), "server must wait for the active request");
            provider.finish.notify_one();
            assert_eq!(request.await.expect("request task"), serde_json::json!({"quotes": [{"symbol": "AAPL"}]}));
            server.await.expect("server task");
        })
        .await
        .expect("shutdown deadline");
    }
}
