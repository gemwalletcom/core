use crate::models::transaction::Transaction;
use chrono::{TimeZone, Utc};
use primitives::{chain::Chain, transaction_utxo::TransactionUtxoInput, TransactionState, TransactionType};
use std::error::Error;

pub fn map_transaction_broadcast(hash: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    if hash.is_empty() {
        Err("Empty transaction hash".into())
    } else {
        Ok(hash)
    }
}

pub fn map_transactions(chain: Chain, transactions: Vec<Transaction>) -> Vec<primitives::Transaction> {
    transactions.into_iter().flat_map(|x| map_transaction(chain, &x)).collect()
}

pub fn map_transaction(chain: Chain, transaction: &Transaction) -> Option<primitives::Transaction> {
    let inputs: Vec<TransactionUtxoInput> = transaction
        .vin
        .iter()
        .filter(|i| i.is_address)
        .map(|input| TransactionUtxoInput {
            address: input.addresses.clone().unwrap().first().unwrap().to_string(),
            value: input.value.clone(),
        })
        .collect();

    let outputs: Vec<TransactionUtxoInput> = transaction
        .vout
        .iter()
        .filter(|o| o.is_address)
        .map(|output| TransactionUtxoInput {
            address: output.addresses.clone().unwrap_or_default().first().unwrap().to_string(),
            value: output.value.clone(),
        })
        .collect();

    if inputs.is_empty() || outputs.is_empty() {
        return None;
    }
    let created_at = Utc.timestamp_opt(transaction.block_time, 0).single()?;

    let transaction = primitives::Transaction::new_with_utxo(
        transaction.txid.clone(),
        chain.as_asset_id(),
        TransactionType::Transfer,
        TransactionState::Confirmed,
        transaction.fees.clone(),
        chain.as_asset_id(),
        "0".to_string(),
        None,
        inputs.into(),
        outputs.into(),
        None,
        created_at,
    );

    Some(transaction)
}
