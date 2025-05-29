use chrono::DateTime;
use primitives::{chain::Chain, AssetId, Transaction, TransactionState, TransactionType};

use super::model::Transaction as XrpTransaction;

const RESULT_SUCCESS: &str = "tesSUCCESS";
const TRANSACTION_TYPE_PAYMENT: &str = "Payment";

pub struct XRPMapper;

impl XRPMapper {
    pub fn map_transaction(chain: Chain, transaction: XrpTransaction, block_number: i64, block_timestamp: i64) -> Option<Transaction> {
        if transaction.transaction_type == TRANSACTION_TYPE_PAYMENT && transaction.memos.unwrap_or_default().is_empty() {
            let memo = transaction.destination_tag.map(|x| x.to_string());
            let value = transaction.amount.clone()?.as_value_string()?;
            let token_id = transaction.amount?.token_id();
            let asset_id = AssetId::from(chain, token_id);
            let created_at = DateTime::from_timestamp(block_timestamp, 0)?;

            let state = if transaction.meta.result == RESULT_SUCCESS {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };
            // add check for delivered amount, for success it should be equal to amount
            let transaction = Transaction::new(
                transaction.hash,
                asset_id.clone(),
                transaction.account.unwrap_or_default(),
                transaction.destination.unwrap_or_default(),
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                transaction.sequence.to_string(),
                transaction.fee.unwrap_or_default(),
                chain.as_asset_id(),
                value,
                memo,
                None,
                created_at,
            );
            return Some(transaction);
        }
        None
    }
}
