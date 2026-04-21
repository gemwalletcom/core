use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct Price {
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HermesResponse {
    pub parsed: Vec<ParsedPriceFeed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedPriceFeed {
    pub id: String,
    pub price: PriceData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceData {
    #[serde(deserialize_with = "serde_serializers::deserialize_u64_from_str")]
    pub price: u64,
    pub expo: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceFeed {
    pub id: String,
}
