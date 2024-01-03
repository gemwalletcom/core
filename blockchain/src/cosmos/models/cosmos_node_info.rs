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
