use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotTransactionMaterial {
    pub at: PolkadotTransactionMaterialBlock,
    pub genesis_hash: String,
    pub chain_name: String,
    pub spec_name: String,
    pub spec_version: String,
    pub tx_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotTransactionMaterialBlock {
    pub height: String,
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
