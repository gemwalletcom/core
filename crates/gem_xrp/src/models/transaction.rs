use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPTransactionBroadcast {
    pub accepted: Option<bool>,
    pub engine_result_message: Option<String>,
    pub error_exception: Option<String>,
    pub tx_json: Option<XRPTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPTransaction {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPTransactionStatus {
    pub status: String,
}
