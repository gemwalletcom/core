#[typeshare]
struct CosmosBroadcastResponse {
    tx_response: CosmosBroadcastResult,
}

#[typeshare]
struct CosmosBroadcastResult {
    txhash: String,
    code: i32,
    raw_log: String,
}

#[typeshare]
struct CosmosTransactionResponse {
    tx_response: CosmosTransactionDataResponse,
}

#[typeshare]
struct CosmosTransactionDataResponse {
    txhash: String,
    code: i32,
}
