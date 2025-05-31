use anyhow::{Context, Result};
use axum::routing::{get, Router};
use futures::pin_mut;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod models;
mod services;
mod websocket;

use services::{DataService, MockDataGenerator};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .context("Failed to parse PORT environment variable")?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // Initialize services
    let data_service = Arc::new(DataService::new());
    let mock_generator = MockDataGenerator::default();

    // Start mock data generation
    let kline_service_clone = data_service.clone();
    tokio::spawn(async move {
        let stream = mock_generator.generate_transaction_stream("DOGE".to_string(), 100);
        pin_mut!(stream);

        while let Some(result) = stream.next().await {
            match result {
                Ok(transaction) => {
                    if let Err(err) = kline_service_clone.process_transaction(&transaction) {
                        tracing::error!("Failed to process transaction: {:#}", err);
                    }
                }
                Err(err) => {
                    tracing::error!("Failed to generate transaction: {:#}", err);
                }
            }
        }
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    // Build the router
    let router = Router::new()
        .route("/health", get(api::health_check))
        .route("/api/v1/klines/{symbol}", get(api::get_klines))
        .route("/ws/klines/{symbol}/{interval}", get(websocket::ws_kline_handler))
        .route("/ws/transactions/{symbol}", get(websocket::ws_transaction_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(data_service);

    tracing::info!("Starting server at {}", addr);

    // Start the server
    axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .context("Failed to bind to address")?,
        router,
    )
    .await
    .context("Server error")?;

    Ok(())
}
