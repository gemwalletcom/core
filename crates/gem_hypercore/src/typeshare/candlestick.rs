use chrono::{DateTime, Utc};
use primitives::chart::ChartCandleStick;
use serde::{Deserialize, Serialize};

use crate::typeshare::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreCandlestick {
    pub t: UInt64, // Open time (timestamp in milliseconds)
    pub o: String, // Open price
    pub h: String, // High price
    pub l: String, // Low price
    pub c: String, // Close price
    pub v: String, // Volume
}

impl From<HypercoreCandlestick> for ChartCandleStick {
    fn from(candlestick: HypercoreCandlestick) -> Self {
        ChartCandleStick {
            date: DateTime::from_timestamp(candlestick.t as i64 / 1000, 0).unwrap_or(Utc::now()),
            open: candlestick.o.parse().unwrap_or(0.0),
            high: candlestick.h.parse().unwrap_or(0.0),
            low: candlestick.l.parse().unwrap_or(0.0),
            close: candlestick.c.parse().unwrap_or(0.0),
            volume: candlestick.v.parse().unwrap_or(0.0),
        }
    }
}
