#[typeshare]
struct CosmosNodeInfo {
    network: String,
}

#[typeshare]
struct CosmosNodeInfoResponse {
    default_node_info: CosmosNodeInfo,
}
