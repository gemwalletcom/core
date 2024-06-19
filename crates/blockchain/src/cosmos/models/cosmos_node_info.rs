#[typeshare]
struct CosmosBlockResponse {
    block: CosmosBlock,
}

#[typeshare]
struct CosmosBlock {
    header: CosmosHeader,
}

#[typeshare]
struct CosmosHeader {
    chain_id: String,
}

#[typeshare]
struct CosmosNodeInfoResponse {
    default_node_info: CosmosNodeInfo,
}

#[typeshare]
struct CosmosNodeInfo {
    network: String,
}
