use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub from_asset: String,
    pub to_asset: String,
    pub amount: String,
    pub destination: String,
    pub affiliate: String,
    pub affiliate_bps: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub expected_amount_out: String,
    pub inbound_address: Option<String>,
    pub memo: String,
}
