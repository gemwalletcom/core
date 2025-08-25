use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UTXOS<T> {
    pub utxos: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UTXO {
    pub address: String,
    pub tx_hash: String,
    pub index: i32,
    pub value: String,
}

impl From<UTXO> for primitives::UTXO {
    fn from(utxo: UTXO) -> Self {
        primitives::UTXO {
            transaction_id: utxo.tx_hash,
            vout: utxo.index,
            value: utxo.value,
            address: utxo.address,
        }
    }
}
