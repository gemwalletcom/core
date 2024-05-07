use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Row, Clone)]
pub struct GetChart {
    pub price: f64,
    pub date: i32,
}

#[derive(Debug, Serialize, Deserialize, Row, Clone)]
pub struct CreateChart {
    pub coin_id: String,
    pub price: f32,
    pub ts: u32,
}
