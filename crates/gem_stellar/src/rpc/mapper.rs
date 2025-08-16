use chrono::DateTime;
use number_formatter::BigNumberFormatter;
use primitives::{chain::Chain, AssetBalance, AssetId, Balance, Transaction, TransactionType};

use crate::typeshare::account::StellarAccount;

use super::model::{Payment, TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};

pub struct StellarMapper;

impl StellarMapper {
    pub fn map_transactions(chain: Chain, transactions: Vec<Payment>) -> Vec<Transaction> {
        transactions.into_iter().flat_map(|x| StellarMapper::map_transaction(chain, x)).collect()
    }

    pub fn map_token_balances(chain: Chain, account: &StellarAccount, token_ids: &[String]) -> Vec<AssetBalance> {
        let mut result = Vec::new();
        for token_id in token_ids {
            if let Some(balance) = account.balances.iter().find(|b| {
                if let (Some(asset_issuer), Some(asset_code)) = (&b.asset_issuer, &b.asset_code) {
                    let balance_token_id = format!("{}-{}", asset_code, asset_issuer);
                    balance_token_id == *token_id && b.asset_type != "native"
                } else {
                    false
                }
            }) {
                if let Ok(amount) = BigNumberFormatter::value_from_amount(&balance.balance, 7) {
                    let asset_id = AssetId::from_token(chain, token_id);
                    let balance_obj = Balance::coin_balance(amount);
                    result.push(AssetBalance::new_with_active(asset_id, balance_obj, true));
                }
            } else {
                let asset_id = AssetId::from_token(chain, token_id);
                let balance_obj = Balance::coin_balance("0".to_string());
                result.push(AssetBalance::new_with_active(asset_id, balance_obj, false));
            }
        }
        result
    }

    pub fn is_token_address(token_id: &str) -> bool {
        token_id.len() > 32 && token_id.contains('-')
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

    pub fn map_balances(chain: Chain, account: StellarAccount) -> Vec<AssetBalance> {
        let mut balances = Vec::new();
        
        for balance in account.balances {
            match balance.asset_type.as_str() {
                "native" => {
                    // Native XLM balance
                    if let Ok(value) = BigNumberFormatter::value_from_amount(&balance.balance, 7) {
                        let balance_obj = Balance::coin_balance(value);
                        balances.push(AssetBalance::new_with_active(chain.as_asset_id(), balance_obj, true));
                    }
                }
                "credit_alphanum4" | "credit_alphanum12" => {
                    // Token balances  
                    if let (Some(asset_issuer), Some(asset_code)) = (&balance.asset_issuer, &balance.asset_code) {
                        let token_id = format!("{}-{}", asset_code, asset_issuer);
                        let asset_id = AssetId::from_token(chain, &token_id);
                        if let Ok(value) = BigNumberFormatter::value_from_amount(&balance.balance, 7) {
                            let balance_obj = Balance::coin_balance(value);
                            balances.push(AssetBalance::new_with_active(asset_id, balance_obj, true));
                        }
                    }
                }
                _ => {
                    // Ignore other asset types
                }
            }
        }
        
        balances
    }
}
