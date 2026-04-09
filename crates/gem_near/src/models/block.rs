use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub hash: String,
    pub height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub chain_id: String,
    pub sync_info: NodeSyncInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSyncInfo {
    pub latest_block_height: u64,
    pub syncing: bool,
}
