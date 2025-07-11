use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
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
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PolkadotTransactionMaterialBlock {
    pub height: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PolkadotTransactionPayload {
    pub tx: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct PolkadotTransactionBroadcast {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct PolkadotTransactionBroadcastError {
    pub error: String,
    pub cause: String,
}
