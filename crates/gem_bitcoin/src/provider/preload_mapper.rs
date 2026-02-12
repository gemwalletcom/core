use std::error::Error;

use primitives::{TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use crate::models::{BitcoinNodeInfo, BitcoinUTXO};

pub fn map_transaction_preload(utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> TransactionLoadMetadata {
    let utxos = map_utxos(utxos, input.sender_address.clone());
    TransactionLoadMetadata::Bitcoin { utxos }
}

pub fn map_utxos(utxos: Vec<BitcoinUTXO>, address: String) -> Vec<UTXO> {
    utxos
        .into_iter()
        .map(|utxo| UTXO {
            transaction_id: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
            address: address.clone(),
        })
        .collect()
}

pub fn map_transaction_preload_zcash(
    node_info: BitcoinNodeInfo,
    utxos: Vec<BitcoinUTXO>,
    input: TransactionPreloadInput,
) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
    let utxos = map_utxos(utxos, input.sender_address.clone());
    let branch_id = node_info.backend.consensus.ok_or("Branch ID not found")?.chaintip;
    Ok(TransactionLoadMetadata::Zcash { utxos, branch_id })
}
