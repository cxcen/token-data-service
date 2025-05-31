use crate::models::KLineInterval;
use crate::services::DataService;
use anyhow::{Context, Result};
use axum::{
    extract::{Path, State, WebSocketUpgrade, ws::Utf8Bytes},
    response::{IntoResponse, Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::{select, time};

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn ws_kline_handler(
    Path((symbol, interval)): Path<(String, String)>,
    State(kline_service): State<Arc<DataService>>,
    ws: WebSocketUpgrade,
) -> Response {
    let interval = match KLineInterval::from_str(&interval) {
        Some(interval) => interval,
        None => return (axum::http::StatusCode::BAD_REQUEST, "Invalid interval").into_response(),
    };

    let symbol = symbol.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, kline_service, symbol, interval))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    kline_service: Arc<DataService>,
    symbol: String,
    interval: KLineInterval,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = kline_service.subscribe();
    let mut ping_interval = time::interval(PING_INTERVAL);
    let mut last_ping_time = None;

    loop {
        select! {
            // Handle incoming WebSocket messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Pong(_))) => {
                        last_ping_time = None;
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            // Handle K-line updates
            Ok(kline) = rx.recv() => {
                if kline.symbol == symbol && kline.interval == interval {
                    let msg = json!({
                        "typ": "kline",
                        "data": kline
                    });

                    if let Ok(text) = serde_json::to_string(&msg).context("Failed to serialize kline message") {
                        let message = axum::extract::ws::Message::Text(text.into());
                        if sender.send(message).await.is_err() {
                            break;
                        }
                    }
                }
            }

            // Send periodic pings
            _ = ping_interval.tick() => {
                // If we haven't received a pong since our last ping, disconnect
                if last_ping_time.is_some() {
                    break;
                }

                let ping_message = axum::extract::ws::Message::Ping(Vec::new().into());
                if sender.send(ping_message).await.is_err() {
                    break;
                }
                last_ping_time = Some(time::Instant::now());
            }
        }

        // Check ping timeout
        if let Some(ping_time) = last_ping_time {
            if ping_time.elapsed() > PING_TIMEOUT {
                break;
            }
        }
    }
} 