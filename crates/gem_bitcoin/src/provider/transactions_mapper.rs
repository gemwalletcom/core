use crate::models::{Address, Transaction};
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
            address: Address::new(input.addresses.clone().unwrap().first().unwrap(), chain).short().to_string(),
            value: input.value.clone(),
        })
        .collect();

    let outputs: Vec<TransactionUtxoInput> = transaction
        .vout
        .iter()
        .filter(|o| o.is_address)
        .map(|output| TransactionUtxoInput {
            address: Address::new(output.addresses.clone().unwrap_or_default().first().unwrap(), chain)
                .short()
                .to_string(),
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
            vin: vec![Input::mock()],
            vout: vec![Output::mock()],
            ..Transaction::mock()
        };

        let result = map_transaction(Chain::Bitcoin, &transaction);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.id.to_string(), "bitcoin_abc123");
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

    #[test]
    fn test_map_transaction_with_address_prefix() {
        let transaction = Transaction {
            txid: "def456".to_string(),
            vin: vec![Input {
                addresses: Some(vec!["bitcoincash:qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q".to_string()]),
                ..Input::mock()
            }],
            vout: vec![Output {
                addresses: Some(vec!["bitcoincash:qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65".to_string()]),
                ..Output::mock()
            }],
            ..Transaction::mock()
        };

        let result = map_transaction(Chain::BitcoinCash, &transaction);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.id.to_string(), "bitcoincash_def456");
        let utxo_inputs = result.utxo_inputs.as_ref().expect("expected at least one input");
        assert_eq!(utxo_inputs.len(), 1);
        assert_eq!(utxo_inputs[0].address, "qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q");
        let utxo_outputs = result.utxo_outputs.as_ref().expect("expected at least one output");
        assert_eq!(utxo_outputs.len(), 1);
        assert_eq!(utxo_outputs[0].address, "qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65");
    }
}
