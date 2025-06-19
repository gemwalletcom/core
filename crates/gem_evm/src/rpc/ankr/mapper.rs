use crate::{
    ethereum_address_checksum,
    rpc::{
        ankr::Transaction,
        mapper::{FUNCTION_ERC20_TRANSFER, INPUT_0X},
    },
};
use chrono::DateTime;
use primitives::Chain;

pub struct AnkrMapper {}

impl AnkrMapper {
    pub fn map_transactions(transactions: Vec<Transaction>, chain: Chain) -> Vec<primitives::Transaction> {
        transactions.into_iter().flat_map(|x| AnkrMapper::map_transaction(x, chain)).collect()
    }

    pub fn map_transaction(transaction: Transaction, chain: Chain) -> Option<primitives::Transaction> {
        let fee = transaction.gas_price.clone() * transaction.gas_used.clone();
        let created_at = DateTime::from_timestamp(transaction.timestamp.to_string().parse::<i64>().ok()?, 0)?;
        let from = ethereum_address_checksum(&transaction.from).ok()?;
        let to = ethereum_address_checksum(&transaction.to).ok()?;
        let status = if transaction.status == "0x1" {
            primitives::TransactionState::Confirmed
        } else {
            primitives::TransactionState::Failed
        };

        match transaction.input.as_str() {
            INPUT_0X => Some(primitives::Transaction::new(
                transaction.hash,
                chain.as_asset_id(),
                from,
                to,
                None,
                primitives::TransactionType::Transfer,
                status,
                fee.to_string(),
                chain.as_asset_id(),
                transaction.value.to_string(),
                None,
                None,
                created_at,
            )),
            FUNCTION_ERC20_TRANSFER => None, //TODO: ERC20 Transfer
            _ => Some(primitives::Transaction::new(
                transaction.hash,
                chain.as_asset_id(),
                from,
                to.clone(),
                Some(to.clone()),
                primitives::TransactionType::SmartContractCall,
                status,
                fee.to_string(),
                chain.as_asset_id(),
                transaction.value.to_string(),
                None,
                None,
                created_at,
            )),
        }
    }
}
