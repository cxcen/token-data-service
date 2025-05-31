use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLine {
    pub symbol: String,
    pub interval: KLineInterval,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub is_closed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KLineInterval {
    #[serde(rename = "1s")]
    OneSecond,
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
}

impl KLineInterval {
    pub fn as_seconds(&self) -> i64 {
        match self {
            KLineInterval::OneSecond => 1,
            KLineInterval::OneMinute => 60,
            KLineInterval::FiveMinutes => 300,
            KLineInterval::FifteenMinutes => 900,
            KLineInterval::OneHour => 3600,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1s" => Some(Self::OneSecond),
            "1m" => Some(Self::OneMinute),
            "5m" => Some(Self::FiveMinutes),
            "15m" => Some(Self::FifteenMinutes),
            "1h" => Some(Self::OneHour),
            _ => None,
        }
    }
}

impl KLine {
    pub fn new(symbol: String, interval: KLineInterval, open_time: DateTime<Utc>, price: Decimal) -> Self {
        let close_time = open_time + chrono::Duration::seconds(interval.as_seconds());
        Self {
            symbol,
            interval,
            open_time,
            close_time,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: Decimal::ZERO,
            is_closed: false,
        }
    }

    pub fn update(&mut self, price: Decimal, volume: Decimal) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
    }

    pub fn close(&mut self) {
        self.is_closed = true;
    }
} 