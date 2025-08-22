use num_bigint::BigUint;
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
    #[serde(deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub fee_charged: BigUint,
    pub hash: String,
}
