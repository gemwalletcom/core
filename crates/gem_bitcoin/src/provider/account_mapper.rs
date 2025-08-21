use crate::models::transaction::BitcoinUTXO;
use primitives::UTXO;

pub fn map_utxos(utxos: Vec<BitcoinUTXO>, address: &str) -> Vec<UTXO> {
    utxos
        .into_iter()
        .map(|utxo| UTXO {
            transaction_id: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
            address: address.to_string(),
        })
        .collect()
}
