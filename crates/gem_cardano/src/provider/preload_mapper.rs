use crate::models::utxo::CardanoUTXO;
use primitives::{TransactionLoadMetadata, TransactionPreloadInput, UTXO};

pub fn map_transaction_preload(utxos: Vec<CardanoUTXO>, _input: TransactionPreloadInput) -> TransactionLoadMetadata {
    let mapped_utxos: Vec<UTXO> = utxos.into_iter().map(UTXO::from).collect();
    TransactionLoadMetadata::Cardano { utxos: mapped_utxos }
}