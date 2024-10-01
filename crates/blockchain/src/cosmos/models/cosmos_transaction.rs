#[typeshare(swift = "Sendable")]
struct CosmosBroadcastResponse {
    tx_response: CosmosBroadcastResult,
}

#[typeshare(swift = "Sendable")]
struct CosmosBroadcastResult {
    txhash: String,
    code: i32,
    raw_log: String,
}

#[typeshare(swift = "Sendable")]
struct CosmosTransactionResponse {
    tx_response: CosmosTransactionDataResponse,
}

#[typeshare(swift = "Sendable")]
struct CosmosTransactionDataResponse {
    txhash: String,
    code: i32,
}
