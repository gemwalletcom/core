use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookResponse {
    pub levels: Vec<Vec<OrderbookLevel>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookLevel {
    pub px: String,
    pub sz: String,
}
