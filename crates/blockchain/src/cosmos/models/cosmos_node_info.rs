#[typeshare(swift = "Sendable")]
struct CosmosBlockResponse {
    block: CosmosBlock,
}

#[typeshare(swift = "Sendable")]
struct CosmosBlock {
    header: CosmosHeader,
}

#[typeshare(swift = "Sendable")]
struct CosmosHeader {
    chain_id: String,
    height: String,
}

#[typeshare(swift = "Sendable")]
struct CosmosNodeInfoResponse {
    default_node_info: CosmosNodeInfo,
}

#[typeshare(swift = "Sendable")]
struct CosmosNodeInfo {
    network: String,
}
