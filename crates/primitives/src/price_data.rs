use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub all_time_high: f64,
    pub all_time_high_date: Option<DateTime<Utc>>,
    pub all_time_low: f64,
    pub all_time_low_date: Option<DateTime<Utc>>,
    pub market_cap: f64,
    pub market_cap_fdv: f64,
    pub market_cap_rank: i32,
    pub total_volume: f64,
    pub circulating_supply: f64,
    pub total_supply: f64,
    pub max_supply: f64,
    pub last_updated_at: DateTime<Utc>,
}
