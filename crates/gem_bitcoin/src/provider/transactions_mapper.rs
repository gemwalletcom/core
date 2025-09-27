use crate::models::transaction::Transaction;
use chrono::{TimeZone, Utc};
use primitives::{TransactionState, TransactionType, chain::Chain, transaction_utxo::TransactionUtxoInput};
use std::error::Error;

pub fn map_transaction_broadcast(hash: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    if hash.is_empty() { Err("Empty transaction hash".into()) } else { Ok(hash) }
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
        transaction.value.clone(),
        None,
        inputs.into(),
        outputs.into(),
        None,
        created_at,
    );

    Some(transaction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{Input, Output};

    #[test]
    fn test_map_transaction() {
        let transaction = Transaction {
            txid: "abc123".to_string(),
            value: "100000".to_string(),
            value_in: "105000".to_string(),
            fees: "5000".to_string(),
            block_time: 1640995200,
            block_height: 700000,
            vin: vec![Input {
                is_address: true,
                addresses: Some(vec!["bc1qinput".to_string()]),
                value: "105000".to_string(),
                n: 0,
                tx_id: Some("prev_tx".to_string()),
                vout: Some(0),
            }],
            vout: vec![Output {
                is_address: true,
                addresses: Some(vec!["bc1qoutput".to_string()]),
                value: "100000".to_string(),
                n: 0,
            }],
        };

        let result = map_transaction(Chain::Bitcoin, &transaction);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.hash, "abc123");
        assert_eq!(result.value, "100000");
        assert_eq!(result.fee, "5000");
        assert_eq!(result.transaction_type, TransactionType::Transfer);
        assert_eq!(result.state, TransactionState::Confirmed);
        let utxo_inputs = result.utxo_inputs.as_ref().expect("expected at least one input");
        assert_eq!(utxo_inputs.len(), 1);
        assert_eq!(utxo_inputs[0].address, "bc1qinput");
        let utxo_outputs = result.utxo_outputs.as_ref().expect("expected at least one output");
        assert_eq!(utxo_outputs.len(), 1);
        assert_eq!(utxo_outputs[0].address, "bc1qoutput");
    }
}
