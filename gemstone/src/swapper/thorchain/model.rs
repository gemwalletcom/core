use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

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
    pub inbound_address: Option<String>,
    pub router: Option<String>,
    pub fees: QuoteFees,
    pub total_swap_seconds: u32,
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

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct InboundAddress {
    pub chain: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub gas_rate: BigInt,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub outbound_fee: BigInt,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub dust_threshold: BigInt,
}
