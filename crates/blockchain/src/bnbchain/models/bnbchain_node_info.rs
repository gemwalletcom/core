#[typeshare]
struct BNBChainNodeInfoResponse {
    #[serde(rename = "nodeInfo")]
    node_info: BNBChainNodeInfo,
    #[serde(rename = "syncInfo")]
    sync_info: BNBChainSyncInfo,
}

#[typeshare]
struct BNBChainSyncInfo {
    #[serde(rename = "catchingUp")]
    catching_up: bool,
}

#[typeshare]
struct BNBChainNodeInfo {
    network: String,
}