use primitives::{TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use crate::models::BitcoinUTXO;

pub fn map_transaction_preload(utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> TransactionLoadMetadata {
    let utxos = utxos
            .into_iter()
            .map(|utxo| UTXO {
                transaction_id: utxo.txid,
                vout: utxo.vout,
                value: utxo.value,
                address: input.sender_address.clone(),
            })
            .collect();

    TransactionLoadMetadata::Bitcoin { utxos }
}