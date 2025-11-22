use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub coin_id: String,
    pub price: f64,
    pub created_at: DateTime<Utc>,
}
