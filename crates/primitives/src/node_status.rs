use crate::UInt64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub chain_id: String,
    pub latest_block_number: UInt64,
    pub latency_ms: UInt64,
}
