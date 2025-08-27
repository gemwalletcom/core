use crate::models::utxo::UTXO as CardanoUTXO;
use primitives::{TransactionLoadMetadata, TransactionPreloadInput};

pub fn map_transaction_preload(utxos: Vec<CardanoUTXO>, _input: TransactionPreloadInput) -> TransactionLoadMetadata {
    TransactionLoadMetadata::Cardano {
        utxos: utxos.into_iter().map(primitives::UTXO::from).collect(),
    }
}
