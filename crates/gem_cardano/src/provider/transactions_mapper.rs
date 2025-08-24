use std::error::Error;
use chrono::DateTime;
use primitives::{chain::Chain, transaction_utxo::TransactionUtxoInput, TransactionDirection, TransactionType};
use crate::models::{Block, Transaction};

pub fn map_transaction_broadcast(hash: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    if hash.is_empty() {
        Err("Empty transaction hash".into())
    } else {
        Ok(hash)
    }
}

pub fn map_transaction(chain: Chain, block: &Block, transaction: &Transaction) -> Option<primitives::Transaction> {
    let inputs: Vec<TransactionUtxoInput> = transaction
        .inputs
        .iter()
        .map(|x| TransactionUtxoInput {
            address: x.address.clone(),
            value: x.value.clone(),
        })
        .collect();

    let outputs: Vec<TransactionUtxoInput> = transaction
        .outputs
        .iter()
        .map(|x| TransactionUtxoInput {
            address: x.address.clone(),
            value: x.value.clone(),
        })
        .collect();

    if inputs.is_empty() || outputs.is_empty() {
        return None;
    }
    let created_at = DateTime::parse_from_rfc3339(&block.forged_at).ok()?.into();

    let transaction = primitives::Transaction::new_with_utxo(
        transaction.hash.clone(),
        chain.as_asset_id(),
        None,
        None,
        None,
        TransactionType::Transfer,
        primitives::TransactionState::Confirmed,
        transaction.fee.clone(),
        chain.as_asset_id(),
        "0".to_string(),
        None,
        TransactionDirection::SelfTransfer,
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
    use crate::models::{Input, Output};

    #[test]
    fn test_map_transaction_broadcast() {
        let hash = "test_hash_123".to_string();
        let result = map_transaction_broadcast(hash).unwrap();
        assert_eq!(result, "test_hash_123");
    }

    #[test]
    fn test_map_transaction() {
        let block = Block {
            number: 123,
            hash: "block_hash".to_string(),
            forged_at: "2023-01-01T00:00:00Z".to_string(),
            transactions: vec![],
        };

        let transaction = Transaction {
            hash: "tx_hash".to_string(),
            inputs: vec![Input {
                address: "addr1".to_string(),
                value: "1000".to_string(),
            }],
            outputs: vec![Output {
                address: "addr2".to_string(),
                value: "900".to_string(),
            }],
            fee: "100".to_string(),
        };

        let result = map_transaction(Chain::Cardano, &block, &transaction);
        assert!(result.is_some());
        
        let mapped_tx = result.unwrap();
        assert_eq!(mapped_tx.hash, "tx_hash");
        assert_eq!(mapped_tx.fee, "100");
    }
}
