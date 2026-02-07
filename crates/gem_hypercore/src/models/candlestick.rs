use chrono::{DateTime, Utc};
use primitives::chart::{ChartCandleStick, ChartCandleUpdate};
use serde::{Deserialize, Serialize};

use crate::models::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    pub t: UInt64, // Open time (timestamp in milliseconds)
    pub s: String, // Symbol (coin)
    pub i: String, // Interval
    pub o: String, // Open price
    pub h: String, // High price
    pub l: String, // Low price
    pub c: String, // Close price
    pub v: String, // Volume
}

impl From<&Candlestick> for ChartCandleStick {
    fn from(c: &Candlestick) -> Self {
        ChartCandleStick {
            date: DateTime::from_timestamp(c.t as i64 / 1000, 0).unwrap_or(Utc::now()),
            open: c.o.parse().unwrap_or(0.0),
            high: c.h.parse().unwrap_or(0.0),
            low: c.l.parse().unwrap_or(0.0),
            close: c.c.parse().unwrap_or(0.0),
            volume: c.v.parse().unwrap_or(0.0),
        }
    }
}

impl From<Candlestick> for ChartCandleUpdate {
    fn from(c: Candlestick) -> Self {
        ChartCandleUpdate {
            coin: c.s.clone(),
            interval: c.i.clone(),
            candle: ChartCandleStick::from(&c),
        }
    }
}
