use num_bigint::BigUint;
use primitives::{AssetId, Chain};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::UInt64;
use crate::models::token::{BigInt, TokenBalance, TokenBalanceChange};

#[derive(Serialize, Deserialize)]
pub struct SolanaTransaction {
    pub meta: SolanaTransactionMeta,
    pub slot: UInt64,
}

#[derive(Serialize, Deserialize)]
pub struct SolanaTransactionMeta {
    pub err: Option<SolanaTransactionError>,
}

#[derive(Serialize, Deserialize)]
pub struct SolanaTransactionError {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalance>,
    pub post_token_balances: Vec<TokenBalance>,
}

impl Meta {
    pub fn get_pre_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.pre_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }

    pub fn get_post_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.post_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }

    pub fn get_pre_token_balance_by_owner(&self, owner: &str) -> Vec<TokenBalance> {
        self.pre_token_balances.iter().filter(|b| b.owner == owner).cloned().collect()
    }

    pub fn get_post_token_balance_by_owner(&self, owner: &str) -> Vec<TokenBalance> {
        self.post_token_balances.iter().filter(|b| b.owner == owner).cloned().collect()
    }

    pub fn get_token_balance_changes_by_owner(&self, owner: &str) -> Vec<TokenBalanceChange> {
        let pre_balances: HashMap<_, _> = self
            .pre_token_balances
            .iter()
            .filter(|b| b.owner == owner)
            .map(|b| (b.mint.clone(), b.get_amount()))
            .collect();

        let post_balances: HashMap<_, _> = self
            .post_token_balances
            .iter()
            .filter(|b| b.owner == owner)
            .map(|b| (b.mint.clone(), b.get_amount()))
            .collect();
        let all_mints: HashSet<_> = pre_balances.keys().chain(post_balances.keys()).cloned().collect();

        all_mints
            .into_iter()
            .filter_map(|mint| {
                let asset_id = AssetId::from_token(Chain::Solana, &mint);
                let pre_amount = pre_balances.get(&mint).cloned().unwrap_or_else(|| BigUint::from(0u64));
                let post_amount = post_balances.get(&mint).cloned().unwrap_or_else(|| BigUint::from(0u64));

                if post_amount > pre_amount {
                    let diff = &post_amount - &pre_amount;
                    Some(TokenBalanceChange {
                        asset_id,
                        amount: BigInt::from_biguint(num_bigint::Sign::Plus, diff),
                    })
                } else if pre_amount > post_amount {
                    let diff = &pre_amount - &post_amount;
                    Some(TokenBalanceChange {
                        asset_id,
                        amount: BigInt::from_biguint(num_bigint::Sign::Minus, diff),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountKey {
    pub pubkey: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub block_time: i64,
    pub signature: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMessage {
    pub account_keys: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}

impl BlockTransaction {
    pub fn fee(&self) -> BigUint {
        BigUint::from(self.meta.fee)
    }

    pub fn get_balance_changes_by_owner(&self, owner: &str) -> TokenBalanceChange {
        // Find all account indices that belong to the owner
        let account_indices: Vec<usize> = self
            .transaction
            .message
            .account_keys
            .iter()
            .enumerate()
            .filter_map(|(i, k)| if k == owner { Some(i) } else { None })
            .collect();

        let (total_pre, total_post) = account_indices.into_iter().fold((0u64, 0u64), |(pre_acc, post_acc), idx| {
            let pre = *self.meta.pre_balances.get(idx).unwrap_or(&0);
            let post = *self.meta.post_balances.get(idx).unwrap_or(&0);
            (pre_acc.wrapping_add(pre), post_acc.wrapping_add(post))
        });

        let (sign, diff) = if total_post > total_pre {
            let diff = total_post - total_pre;
            (num_bigint::Sign::Plus, BigUint::from(diff))
        } else {
            let diff = total_pre - total_post;
            (num_bigint::Sign::Minus, BigUint::from(diff))
        };
        let fee = self.fee();
        let data = if fee > diff { BigUint::from(0u64) } else { diff - fee };

        TokenBalanceChange {
            asset_id: Chain::Solana.as_asset_id(),
            amount: BigInt::from_biguint(sign, data),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactions {
    pub block_time: i64,
    pub transactions: Vec<BlockTransaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleTransaction {
    pub block_time: i64,
    pub meta: Meta,
    pub transaction: Transaction,
}
