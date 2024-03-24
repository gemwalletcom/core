use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedFee {
    pub fee_type: String,
    pub bridge_id: String,
    pub amount: String,
    pub usd_amount: String,
    pub origin_asset: Value,
    pub chain_id: String,
    pub tx_index: i64,
}
