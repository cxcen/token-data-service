use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub symbol: String,
    pub price: Decimal,
    pub volume: Decimal,
    pub timestamp: DateTime<Utc>,
    pub side: TradeSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

impl Transaction {
    pub fn new(symbol: String, price: Decimal, volume: Decimal, side: TradeSide) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            price,
            volume,
            timestamp: Utc::now(),
            side,
        }
    }

    pub fn total_value(&self) -> Decimal {
        self.price * self.volume
    }
} 