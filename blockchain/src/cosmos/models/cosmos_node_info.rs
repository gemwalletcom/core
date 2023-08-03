#[typeshare]
struct CosmosNodeInfo {
    network: String,
}

#[typeshare]
struct CosmosNodeInfoResponse {
    node_info: CosmosNodeInfo,
}
