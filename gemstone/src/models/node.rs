use primitives::NodeStatus;

pub type GemNodeStatus = NodeStatus;

#[uniffi::remote(Record)]
pub struct NodeStatus {
    pub chain_id: String,
    pub latest_block_number: u64,
    pub latency_ms: u64,
}
