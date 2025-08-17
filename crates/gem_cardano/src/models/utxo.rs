use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoUTXOS<T> {
    pub utxos: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoUTXO {
    pub address: String,
    pub tx_hash: String,
    pub index: i32,
    pub value: String,
}
