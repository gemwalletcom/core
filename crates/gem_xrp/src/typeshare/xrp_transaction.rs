use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPTransactionBroadcast {
    pub accepted: Option<bool>,
    pub engine_result_message: Option<String>,
    pub error_exception: Option<String>,
    pub tx_json: Option<XRPTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPTransaction {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPTransactionStatus {
    pub status: String,
}