use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::PriceProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub id: String,
    pub provider: PriceProvider,
    pub provider_price_id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub all_time_high: f64,
    pub all_time_high_date: Option<DateTime<Utc>>,
    pub all_time_low: f64,
    pub all_time_low_date: Option<DateTime<Utc>>,
    pub market_cap_rank: Option<i32>,
    pub last_updated_at: DateTime<Utc>,
}
