use num_bigint::BigInt;
use primitives::swap::SwapStatus;
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
    pub total_swap_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteFees {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub tx: TransactionTx,
    pub observed_tx: TransactionObserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTx {
    pub memo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionObserved {
    pub status: Option<String>,
    pub out_hashes: Option<Vec<String>>,
}

impl TransactionObserved {
    pub fn swap_status(&self) -> SwapStatus {
        match self.status.as_deref() {
            Some("done") => SwapStatus::Completed,
            _ => SwapStatus::Failed, // TODO: Handle refunded status detection later
        }
    }
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
