use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotTransactionMaterial {
    pub at: PolkadotTransactionMaterialBlock,
    pub genesis_hash: String,
    pub chain_name: String,
    pub spec_name: String,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub spec_version: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub tx_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotTransactionMaterialBlock {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub height: u64,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotTransactionPayload {
    pub tx: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotTransactionBroadcastResponse {
    pub hash: Option<String>,
    pub error: Option<String>,
    pub cause: Option<String>,
}
