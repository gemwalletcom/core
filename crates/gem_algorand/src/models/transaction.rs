use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandTransactionParams {
    #[serde(rename = "min-fee")]
    pub min_fee: UInt64,
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,
    #[serde(rename = "genesis-hash")]
    pub genesis_hash: String,
    #[serde(rename = "last-round")]
    pub last_round: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandTransactionBroadcast {
    #[serde(rename = "txId")]
    pub tx_id: Option<String>,
    #[serde(rename = "message")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandTransactionStatus {
    #[serde(rename = "confirmed-round")]
    pub confirmed_round: UInt64,
}
