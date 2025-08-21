use serde::{Deserialize, Serialize};
use primitives::UTXO;

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

impl From<CardanoUTXO> for UTXO {
    fn from(utxo: CardanoUTXO) -> Self {
        UTXO {
            transaction_id: utxo.tx_hash,
            vout: utxo.index,
            value: utxo.value,
            address: utxo.address,
        }
    }
}
