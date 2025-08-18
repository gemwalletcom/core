use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBroadcastRequest {
    pub mode: String,
    pub tx_bytes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBroadcastResponse {
    pub tx_response: Option<CosmosTransactionResult>,
    pub code: Option<i32>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosTransactionResult {
    pub txhash: String,
    pub code: i32,
    pub raw_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosTransactionResponse {
    pub tx_response: CosmosTransactionResult,
}
