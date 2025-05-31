use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::{select, time};
use crate::services::DataService;

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn ws_transaction_handler(
    Path(symbol): Path<String>,
    State(kline_service): State<Arc<DataService>>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_transaction_socket(socket, kline_service, symbol))
}

async fn handle_transaction_socket(
    socket: axum::extract::ws::WebSocket,
    data_service: Arc<DataService>,
    symbol: String,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = data_service.subscribe_transactions();
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

            // Handle transaction updates
            Ok(transaction) = rx.recv() => {
                if transaction.symbol == symbol {
                    let msg = json!({
                        "typ": "transaction",
                        "data": transaction
                    });

                    if let Ok(text) = serde_json::to_string(&msg) {
                        let message = axum::extract::ws::Message::Text(text.into());
                        if sender.send(message).await.is_err() {
                            break;
                        }
                    }
                }
            }

            // Send periodic pings
            _ = ping_interval.tick() => {
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