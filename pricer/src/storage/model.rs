use serde::{Deserialize, Serialize};
use clickhouse::Row;

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct ChartPrice {
    pub coin_id: String,
    pub price: f64,
    pub created_at: u64,
}