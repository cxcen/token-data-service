use crate::models::{KLine, KLineInterval, Transaction};
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

const MAX_HISTORY: usize = 1000;
const BROADCAST_CHANNEL_SIZE: usize = 100;

pub struct KLineService {
    klines: Arc<DashMap<(String, KLineInterval), Vec<KLine>>>,
    current_klines: Arc<DashMap<(String, KLineInterval), KLine>>,
    tx: broadcast::Sender<KLine>,    
}

impl KLineService {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(BROADCAST_CHANNEL_SIZE);
        Self {
            klines: Arc::new(DashMap::new()),
            current_klines: Arc::new(DashMap::new()),
            tx
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<KLine> {
        let rx = self.tx.subscribe();
        info!("receiver_count {}", self.tx.receiver_count());
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
        info!("channle size {}", self.tx.len());
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
    fn test_kline_service() -> Result<()> {
        let service = KLineService::new();
        let symbol = "DOGE".to_string();
        
        // Create a test transaction
        let transaction = Transaction::new(
            symbol.clone(),
            Decimal::new(100, 0),
            Decimal::new(1, 0),
            TradeSide::Buy,
        );

        // Process the transaction
        service.process_transaction(&transaction)?;

        // Verify KLine was created
        let klines = service.get_klines(&symbol, KLineInterval::OneMinute, 1);
        assert!(!klines.is_empty());
        Ok(())
    }
} 