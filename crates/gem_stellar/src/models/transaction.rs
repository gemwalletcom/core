use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransactionBroadcast {
    pub hash: Option<String>,
    #[serde(rename = "title")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransactionStatus {
    pub successful: bool,
    pub fee_charged: String,
}
