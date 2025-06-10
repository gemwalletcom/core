use crate::{Transaction as AptosTransaction, STAKE_DEPOSIT_EVENT};
use chrono::DateTime;
use num_bigint::BigUint;
use primitives::{Chain, Transaction, TransactionState, TransactionType};
use std::str::FromStr;

pub struct AptosMapper;

impl AptosMapper {
    pub fn map_transactions(chain: Chain, transactions: Vec<AptosTransaction>, block_number: i64) -> Vec<Transaction> {
        let mut transactions = transactions
            .into_iter()
            .flat_map(|x| AptosMapper::map_transaction(chain, x, block_number))
            .collect::<Vec<_>>();

        transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        transactions
    }

    pub fn map_transaction(chain: Chain, transaction: AptosTransaction, block_number: i64) -> Option<Transaction> {
        let events = transaction.clone().events.unwrap_or_default();

        if transaction.transaction_type == "user_transaction" && events.len() <= 4 {
            let deposit_event = events.iter().find(|x| x.event_type == STAKE_DEPOSIT_EVENT)?;

            let asset_id = chain.as_asset_id();
            let state = if transaction.success {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };
            let to = &deposit_event.guid.account_address;
            let value = &deposit_event.get_amount()?;
            let gas_used = BigUint::from_str(transaction.gas_used.unwrap_or_default().as_str()).unwrap_or_default();
            let gas_unit_price = BigUint::from_str(transaction.gas_unit_price.unwrap_or_default().as_str()).unwrap_or_default();
            let fee = gas_used * gas_unit_price;
            let created_at = DateTime::from_timestamp_micros(transaction.timestamp as i64)?;

            let transaction = Transaction::new(
                transaction.hash,
                asset_id.clone(),
                transaction.sender.unwrap_or_default(),
                to.clone(),
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                transaction.sequence_number.unwrap_or_default(),
                fee.to_string(),
                asset_id,
                value.clone(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }
        None
    }
}
