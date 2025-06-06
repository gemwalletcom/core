use chrono::Utc;
use primitives::{chain::Chain, transaction_utxo::TransactionInput, TransactionDirection, TransactionType};

use super::model::{Block, Transaction};

pub struct CardanoMapper;

impl CardanoMapper {
    pub fn map_transaction(chain: Chain, block: &Block, transaction: &Transaction) -> Option<primitives::Transaction> {
        let inputs: Vec<TransactionInput> = transaction
            .inputs
            .iter()
            .map(|x| TransactionInput {
                address: x.address.clone(),
                value: x.value.clone(),
            })
            .collect();

        let outputs: Vec<TransactionInput> = transaction
            .outputs
            .iter()
            .map(|x| TransactionInput {
                address: x.address.clone(),
                value: x.value.clone(),
            })
            .collect();

        if inputs.is_empty() || outputs.is_empty() {
            return None;
        }

        let transaction = primitives::Transaction::new_with_utxo(
            transaction.hash.clone(),
            chain.as_asset_id(),
            None,
            None,
            None,
            TransactionType::Transfer,
            primitives::TransactionState::Confirmed,
            block.number.to_string(),
            0.to_string(),
            transaction.fee.clone(),
            chain.as_asset_id(),
            "0".to_string(),
            None,
            TransactionDirection::SelfTransfer,
            inputs.into(),
            outputs.into(),
            None,
            Utc::now(),
        );

        Some(transaction)
    }
}
