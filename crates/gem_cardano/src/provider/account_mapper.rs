use crate::models::utxo::CardanoUTXO;
use primitives::UTXO;

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

pub fn map_utxos(utxos: Vec<CardanoUTXO>) -> Vec<UTXO> {
    utxos.into_iter().map(UTXO::from).collect()
}