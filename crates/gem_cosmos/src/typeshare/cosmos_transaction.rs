use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosBroadcastResponse {
    pub tx_response: CosmosBroadcastResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosBroadcastResult {
    pub txhash: String,
    pub code: i32,
    pub raw_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosTransactionResponse {
    pub tx_response: CosmosTransactionDataResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosTransactionDataResponse {
    pub txhash: String,
    pub code: i32,
}