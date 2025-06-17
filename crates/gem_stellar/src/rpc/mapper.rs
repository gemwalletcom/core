use chrono::DateTime;
use number_formatter::BigNumberFormatter;
use primitives::{chain::Chain, AssetBalance, AssetId, Transaction, TransactionType};

use crate::rpc::model::Account;

use super::model::{Payment, TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};

pub struct StellarMapper;

impl StellarMapper {
    pub fn map_transactions(chain: Chain, transactions: Vec<Payment>) -> Vec<Transaction> {
        transactions.into_iter().flat_map(|x| StellarMapper::map_transaction(chain, x)).collect()
    }

    pub fn map_transaction(chain: Chain, transaction: Payment) -> Option<Transaction> {
        match transaction.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT | TRANSACTION_TYPE_CREATE_ACCOUNT => {
                if transaction.clone().asset_type.unwrap_or_default() == "native"
                    || transaction.clone().payment_type.as_str() == TRANSACTION_TYPE_CREATE_ACCOUNT
                {
                    let created_at = DateTime::parse_from_rfc3339(&transaction.created_at).ok()?.into();

                    return Some(Transaction::new(
                        transaction.clone().transaction_hash,
                        chain.as_asset_id(),
                        transaction.from_address()?,
                        transaction.to_address()?,
                        None,
                        TransactionType::Transfer,
                        transaction.get_state(),
                        "0".to_string(),
                        0.to_string(),
                        "1000".to_string(), // TODO: Calculate from block/transaction
                        chain.as_asset_id(),
                        transaction.get_value()?,
                        transaction.clone().get_memo(),
                        None,
                        created_at,
                    ));
                }

                None
            }
            _ => None,
        }
    }

    pub fn map_balances(chain: Chain, account: Account) -> Vec<AssetBalance> {
        account
            .balances
            .into_iter()
            .filter(|x| x.asset_type == "credit_alphanum4")
            .filter_map(|x| {
                let asset_id = AssetId::from_token(chain, &x.asset_issuer?);
                let value = BigNumberFormatter::value_from_amount(&x.balance, 7)?;
                Some(AssetBalance::new(asset_id, value))
            })
            .collect()
    }
}
