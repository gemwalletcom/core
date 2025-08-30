use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub chain_id: String,
    pub latest_block_number: u64,
    pub latency_ms: u64,
}
