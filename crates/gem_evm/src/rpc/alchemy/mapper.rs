use chrono::DateTime;
use primitives::Chain;

use crate::{ethereum_address_checksum, rpc::alchemy::model::Transaction};

pub struct AlchemyMapper {}

impl AlchemyMapper {
    pub fn map_transactions(transactions: Vec<Transaction>, chain: Chain) -> Vec<primitives::Transaction> {
        transactions.into_iter().flat_map(|x| AlchemyMapper::map_transaction(x, chain)).collect()
    }

    pub fn map_transaction(transaction: Transaction, chain: Chain) -> Option<primitives::Transaction> {
        if transaction.logs.is_empty() && transaction.internal_transactions.is_empty() && transaction.value != "0" {
            let created_at = DateTime::from_timestamp_millis(transaction.block_timestamp as i64)?;
            let fee = (transaction.gas_price * transaction.gas).to_string();
            let contract_address = match &transaction.contract_address {
                Some(contract) if contract == "null" => None,
                Some(contract) => Some(contract.clone()),
                None => None,
            };
            let from = ethereum_address_checksum(&transaction.from_address.clone()).ok()?;
            let to = ethereum_address_checksum(&transaction.to_address.clone()).ok()?;
            let transaction = primitives::Transaction::new(
                transaction.hash,
                chain.as_asset_id(),
                from,
                to,
                contract_address,
                primitives::TransactionType::Transfer,
                primitives::TransactionState::Confirmed,
                fee,
                chain.as_asset_id(),
                transaction.value.clone(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        None
    }
}
