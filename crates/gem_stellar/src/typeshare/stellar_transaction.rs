use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct StellarTransactionBroadcast {
    pub hash: Option<String>,
    #[serde(rename = "title")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct StellarTransactionStatus {
    pub successful: bool,
    pub fee_charged: String,
}