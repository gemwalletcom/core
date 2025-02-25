use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapRequest {
    pub from_asset: String,
    pub to_asset: String,
    pub amount: String,
    pub affiliate: String,
    pub affiliate_bps: i64,
    pub streaming_interval: i64,
    pub streaming_quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapResponse {
    pub expected_amount_out: String,
    pub recommended_min_amount_in: String,
    pub inbound_address: Option<String>,
    pub router: Option<String>,
    pub fees: QuoteFees,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteFees {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub observed_tx: TransactionObserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionObserved {
    pub status: String, // done
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData {
    pub router_address: Option<String>,
    pub inbound_address: Option<String>,
}
