use chrono::DateTime;
use primitives::{chain::Chain, transaction_utxo::TransactionUtxoInput, TransactionDirection, TransactionType};

use super::model::{Block, Transaction};

pub struct CardanoMapper;

impl CardanoMapper {
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
}
