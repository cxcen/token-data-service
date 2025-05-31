use anyhow::Context;
use futures_util::StreamExt;
use data_service::models::{KLine, Transaction};
use serde::Deserialize;
use std::env;
use std::net::SocketAddr;
use tokio::task::JoinSet;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Deserialize)]
struct KLineData {
    typ: String,
    data: KLine,
}
#[derive(Debug, Deserialize)]
struct TxData {
    typ: String,
    data: Transaction,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .context("Failed to parse PORT environment variable")?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let mut tasks = JoinSet::new();
    for _ in 0..10 {
        tasks.spawn(async move {
            let (mut socket, _response) =
                tokio_tungstenite::connect_async(format!("ws://{addr}/ws/klines/DOGE/1m"))
                    .await
                    .unwrap();
            loop {
                let msg = socket.next().await.unwrap().unwrap();
                match msg {
                    Message::Text(msg) => {
                        let data = serde_json::from_slice::<KLineData>(msg.as_ref());
                        println!("{:?}", data);
                    }
                    _ => {}
                }
            }
        });

        tasks.spawn(async move {
            let (mut socket, _response) =
                tokio_tungstenite::connect_async(format!("ws://{addr}/ws/transactions/DOGE"))
                    .await
                    .unwrap();
            loop {
                let msg = socket.next().await.unwrap().unwrap();
                match msg {
                    Message::Text(msg) => {
                        let data = serde_json::from_slice::<TxData>(msg.as_ref());
                        println!("{:?}", data);
                    }
                    _ => {}
                }
            }
        });
    }

    tasks.join_all().await;

    Ok(())
}
