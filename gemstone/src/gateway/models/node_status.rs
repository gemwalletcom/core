use primitives::NodeStatus;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemNodeStatus {
    pub chain_id: String,
    pub latest_block_number: u64,
    pub latency_ms: u64,
}

impl From<NodeStatus> for GemNodeStatus {
    fn from(status: NodeStatus) -> Self {
        Self {
            chain_id: status.chain_id,
            latest_block_number: status.latest_block_number,
            latency_ms: status.latency_ms,
        }
    }
}

impl From<GemNodeStatus> for NodeStatus {
    fn from(status: GemNodeStatus) -> Self {
        Self {
            chain_id: status.chain_id,
            latest_block_number: status.latest_block_number,
            latency_ms: status.latency_ms,
        }
    }
}
