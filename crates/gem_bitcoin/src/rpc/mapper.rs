use chrono::{TimeZone, Utc};
use primitives::{chain::Chain, transaction_utxo::TransactionInput, TransactionDirection, TransactionType};

use super::model::Transaction;

pub struct BitcoinMapper;

impl BitcoinMapper {
    pub fn map_transaction(chain: Chain, transaction: &Transaction) -> Option<primitives::Transaction> {
        let inputs: Vec<TransactionInput> = transaction
            .vin
            .iter()
            .filter(|i| i.is_address)
            .map(|input| TransactionInput {
                address: input.addresses.clone().unwrap().first().unwrap().to_string(),
                value: input.value.clone(),
            })
            .collect();

        let outputs: Vec<TransactionInput> = transaction
            .vout
            .iter()
            .filter(|o| o.is_address)
            .map(|output| TransactionInput {
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
            None,
            None,
            None,
            TransactionType::Transfer,
            primitives::TransactionState::Confirmed,
            transaction.block_height.to_string(),
            0.to_string(),
            transaction.fees.clone(),
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
