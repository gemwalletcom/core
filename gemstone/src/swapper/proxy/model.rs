use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub from_address: String,
    pub from_asset: String,
    pub to_address: String,
    pub to_asset: String,
    pub from_value: String,
    pub referral_address: String,
    pub referral_bps: usize,
    pub slippage_bps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub quote: QuoteRequest,
    pub output_value: String,
    pub output_min_value: String,
    pub route_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
}
