use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBroadcastResponse {
    pub tx_response: CosmosBroadcastResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBroadcastResult {
    pub txhash: String,
    pub code: i32,
    pub raw_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosTransactionResponse {
    pub tx_response: CosmosTransactionDataResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosTransactionDataResponse {
    pub txhash: String,
    pub code: i32,
}
