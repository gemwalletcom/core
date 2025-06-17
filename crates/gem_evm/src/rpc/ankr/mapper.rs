use chrono::DateTime;
use primitives::Chain;

use crate::{ethereum_address_checksum, rpc::ankr::Transaction};

pub struct AnkrMapper {}

impl AnkrMapper {
    pub fn map_transactions(transactions: Vec<Transaction>, chain: Chain) -> Vec<primitives::Transaction> {
        transactions.into_iter().flat_map(|x| AnkrMapper::map_transaction(x, chain)).collect()
    }

    pub fn map_transaction(transaction: Transaction, chain: Chain) -> Option<primitives::Transaction> {
        // TODO Add Support for smart contract calls and ERC20
        if transaction.input != "0x" {
            return None;
        }
        let fee = transaction.gas_price.clone() * transaction.gas_used.clone();
        let created_at = DateTime::from_timestamp(transaction.timestamp.to_string().parse::<i64>().ok()?, 0)?;
        let from = ethereum_address_checksum(&transaction.from).ok()?;
        let to = ethereum_address_checksum(&transaction.to).ok()?;
        let status = if transaction.status == "0x1" {
            primitives::TransactionState::Confirmed
        } else {
            primitives::TransactionState::Failed
        };
        let contract_address = transaction.contract_address.and_then(|x| ethereum_address_checksum(&x).ok());

        let transaction = primitives::Transaction::new(
            transaction.hash,
            chain.as_asset_id(),
            from,
            to,
            contract_address,
            primitives::TransactionType::Transfer,
            status,
            transaction.block_number.to_string(),
            transaction.nonce.to_string(),
            fee.to_string(),
            chain.as_asset_id(),
            transaction.value.to_string(),
            None,
            None,
            created_at,
        );

        Some(transaction)
    }
}
