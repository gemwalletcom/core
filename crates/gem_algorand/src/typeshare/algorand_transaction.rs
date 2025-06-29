use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct AlgorandTransactionParams {
    #[serde(rename = "min-fee")]
    pub min_fee: i32,
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,
    #[serde(rename = "genesis-hash")]
    pub genesis_hash: String,
    #[serde(rename = "last-round")]
    pub last_round: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct AlgorandTransactionBroadcast {
    #[serde(rename = "txId")]
    pub tx_id: Option<String>,
    #[serde(rename = "message")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct AlgorandTransactionStatus {
    #[serde(rename = "confirmed-round")]
    pub confirmed_round: i32,
}