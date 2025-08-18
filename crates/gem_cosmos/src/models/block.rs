use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBlockResponse {
    pub block: CosmosBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBlock {
    pub header: CosmosHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosHeader {
    pub chain_id: String,
    pub height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosNodeInfoResponse {
    pub default_node_info: CosmosNodeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosNodeInfo {
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosSyncing {
    pub syncing: bool,
}
