use crate::models::{TradeSide, Transaction};
use anyhow::{Context, Result};
use async_stream::stream;
use futures::Stream;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rust_decimal::Decimal;
use std::{pin::Pin, str::FromStr};
use tokio::time::{interval, Duration};

pub struct MockDataGenerator {
    base_price: Decimal,
    volatility: f64,
    min_volume: Decimal,
    max_volume: Decimal,
}

impl Default for MockDataGenerator {
    fn default() -> Self {
        Self {
            base_price: Decimal::from_str("100.0").expect("Invalid default base price"),
            volatility: 0.002, // 0.2% volatility
            min_volume: Decimal::from_str("0.1").expect("Invalid default min volume"),
            max_volume: Decimal::from_str("10.0").expect("Invalid default max volume"),
        }
    }
}

impl MockDataGenerator {
    pub fn new(base_price: Decimal, volatility: f64, min_volume: Decimal, max_volume: Decimal) -> Self {
        Self {
            base_price,
            volatility,
            min_volume,
            max_volume,
        }
    }

    pub fn generate_transaction_stream(
        &self,
        symbol: String,
        interval_ms: u64,
    ) -> Pin<Box<dyn Stream<Item = Result<Transaction>> + Send>> {
        let base_price = self.base_price;
        let volatility = self.volatility;
        let min_volume = self.min_volume;
        let max_volume = self.max_volume;

        let mut interval = interval(Duration::from_millis(interval_ms));
        let mut current_price = base_price;
        let mut rng = StdRng::from_os_rng();

        Box::pin(stream! {
            loop {
                interval.tick().await;

                // Generate price movement
                let price_change = (rng.gen::<f64>() - 0.5) * 2.0 * volatility;
                let price_multiplier = Decimal::from_str(&(1.0 + price_change).to_string())
                    .context("Failed to create price multiplier")?;
                current_price *= price_multiplier;

                // Generate random volume
                let volume_range = max_volume - min_volume;
                let random_volume = min_volume + (volume_range * 
                    Decimal::from_str(&rng.random::<f64>().to_string())
                        .context("Failed to generate random volume")?);

                // Randomly choose trade side
                let side = if rng.gen_bool(0.5) {
                    TradeSide::Buy
                } else {
                    TradeSide::Sell
                };

                yield Ok(Transaction::new(
                    symbol.clone(),
                    current_price,
                    random_volume,
                    side,
                ));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_mock_data_generator() {
        let generator = MockDataGenerator::default();
        let mut stream = generator.generate_transaction_stream("DOGE".to_string(), 100);

        let transaction = stream.next().await.unwrap().unwrap();
        assert_eq!(transaction.symbol, "DOGE");
        assert!(transaction.price > Decimal::ZERO);
        assert!(transaction.volume > Decimal::ZERO);
    }
} 