use primitives::{chain::Chain, Transaction, TransactionType};

use super::model::{Block, Payment, TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};

pub struct StellarMapper;

impl StellarMapper {
    pub fn map_transaction(chain: Chain, block: Block, transaction: Payment) -> Option<Transaction> {
        match transaction.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT | TRANSACTION_TYPE_CREATE_ACCOUNT => {
                if transaction.clone().asset_type.unwrap_or_default() == "native"
                    || transaction.clone().payment_type.as_str() == TRANSACTION_TYPE_CREATE_ACCOUNT
                {
                    return Some(Transaction::new(
                        transaction.clone().transaction_hash,
                        chain.as_asset_id(),
                        transaction.clone().from.unwrap_or_default(),
                        transaction.clone().to.unwrap_or_default(),
                        None,
                        TransactionType::Transfer,
                        transaction.get_state(),
                        block.sequence.to_string(),
                        0.to_string(),
                        block.base_fee_in_stroops.to_string(), // TODO: Calculate from block/transaction
                        chain.as_asset_id(),
                        transaction.get_value().unwrap_or("0".to_string()).to_string(),
                        transaction.clone().get_memo(),
                        None,
                        block.closed_at.parse().unwrap_or_default(),
                    ));
                }

                None
            }
            _ => None,
        }
    }
}
