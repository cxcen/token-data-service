use crate::models::{KLine, KLineInterval, Transaction};
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;
use tokio::runtime::Runtime;

const MAX_HISTORY: usize = 1000;
const BROADCAST_CHANNEL_SIZE: usize = 1000;

pub struct DataService {
    klines: Arc<DashMap<(String, KLineInterval), Vec<KLine>>>,
    current_klines: Arc<DashMap<(String, KLineInterval), KLine>>,
    tx: broadcast::Sender<KLine>,
    transaction_tx: broadcast::Sender<Transaction>,
}

impl DataService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);
        let (transaction_tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);
        Self {
            klines: Arc::new(DashMap::new()),
            current_klines: Arc::new(DashMap::new()),
            tx,
            transaction_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<KLine> {
        let rx = self.tx.subscribe();
        info!("receiver_count {}", self.tx.receiver_count());
        rx
    }

    pub fn subscribe_transactions(&self) -> broadcast::Receiver<Transaction> {
        let rx = self.transaction_tx.subscribe();
        info!("transaction_receiver_count {}", self.transaction_tx.receiver_count());
        rx
    }

    pub fn get_klines(&self, symbol: &str, interval: KLineInterval, limit: usize) -> Vec<KLine> {
        self.klines
            .get(&(symbol.to_string(), interval))
            .map(|klines| {
                let start = if klines.len() > limit {
                    klines.len() - limit
                } else {
                    0
                };
                klines[start..].to_vec()
            })
            .unwrap_or_default()
    }

    pub fn process_transaction(&self, transaction: &Transaction) -> Result<()> {
        // Broadcast the transaction first
        self.transaction_tx.send(transaction.clone())
            .context("Failed to broadcast transaction")?;

        for interval in [
            KLineInterval::OneSecond,
            KLineInterval::OneMinute,
            KLineInterval::FiveMinutes,
            KLineInterval::FifteenMinutes,
            KLineInterval::OneHour,
        ] {
            self.update_kline(transaction, interval)
                .with_context(|| format!("Failed to update kline for interval {:?}", interval))?;
        }
        Ok(())
    }

    fn update_kline(&self, transaction: &Transaction, interval: KLineInterval) -> Result<()> {
        let key = (transaction.symbol.clone(), interval);
        let timestamp = transaction.timestamp;
        
        // Get or create current KLine
        let mut current_kline = self.current_klines.entry(key.clone()).or_insert_with(|| {
            let open_time = self.calculate_kline_start(timestamp, interval);
            KLine::new(
                transaction.symbol.clone(),
                interval,
                open_time,
                transaction.price,
            )
        });

        // Check if we need to close the current KLine
        if timestamp >= current_kline.close_time {
            // Close current KLine
            current_kline.close();
            let closed_kline = current_kline.clone();

            // Store in history
            self.klines
                .entry(key.clone())
                .or_default()
                .push(closed_kline.clone());

            // Trim history if needed
            if let Some(mut klines) = self.klines.get_mut(&key) {
                if klines.len() > MAX_HISTORY {
                    let len = klines.len();
                    klines.drain(0..len - MAX_HISTORY);
                }
            }

            // Broadcast the closed KLine
            self.tx.send(closed_kline)
                .context("Failed to broadcast closed KLine")?;

            // Create new KLine
            let new_kline = KLine::new(
                transaction.symbol.clone(),
                interval,
                self.calculate_kline_start(timestamp, interval),
                transaction.price,
            );
            *current_kline = new_kline;
        }

        // Update the current KLine
        current_kline.update(transaction.price, transaction.volume);

        // Broadcast the updated current KLine
        self.tx.send(current_kline.clone())
            .context("Failed to broadcast updated KLine")?;

        Ok(())
    }

    fn calculate_kline_start(&self, timestamp: DateTime<Utc>, interval: KLineInterval) -> DateTime<Utc> {
        let seconds = timestamp.timestamp();
        let interval_seconds = interval.as_seconds();
        let start_seconds = (seconds / interval_seconds) * interval_seconds;
        Utc.timestamp_opt(start_seconds, 0)
            .single()
            .expect("Invalid timestamp calculation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TradeSide;
    use rust_decimal::Decimal;

    #[test]
    fn test_data_service() -> Result<()> {
        let service = DataService::new();
        let symbol = "DOGE".to_string();
        
        let transaction = Transaction::new(
            symbol.clone(),
            Decimal::new(100, 0),
            Decimal::new(1, 0),
            TradeSide::Buy,
        );

        let mut transaction_rx = service.subscribe_transactions();
        
        let mut kline_rx = service.subscribe();

        service.process_transaction(&transaction)?;

        let rt = Runtime::new()?;
        
        rt.block_on(async {
            if let Ok(received_transaction) = transaction_rx.recv().await {
                assert_eq!(received_transaction.symbol, symbol);
                assert_eq!(received_transaction.price, Decimal::new(100, 0));
                assert_eq!(received_transaction.volume, Decimal::new(1, 0));
                assert_eq!(received_transaction.side, TradeSide::Buy);
            } else {
                panic!("Failed to receive transaction");
            }
        });

        rt.block_on(async {
            let mut received_intervals = Vec::new();
            
            // We expect 5 intervals (1s, 1m, 5m, 15m, 1h)
            for _ in 0..5 {
                if let Ok(kline) = kline_rx.recv().await {
                    assert_eq!(kline.symbol, symbol);
                    assert_eq!(kline.open, Decimal::new(100, 0));
                    assert_eq!(kline.high, Decimal::new(100, 0));
                    assert_eq!(kline.low, Decimal::new(100, 0));
                    assert_eq!(kline.close, Decimal::new(100, 0));
                    assert_eq!(kline.volume, Decimal::new(1, 0));
                    received_intervals.push(kline.interval);
                }
            }

            assert!(received_intervals.contains(&KLineInterval::OneSecond));
            assert!(received_intervals.contains(&KLineInterval::OneMinute));
            assert!(received_intervals.contains(&KLineInterval::FiveMinutes));
            assert!(received_intervals.contains(&KLineInterval::FifteenMinutes));
            assert!(received_intervals.contains(&KLineInterval::OneHour));
        });

        Ok(())
    }
} 