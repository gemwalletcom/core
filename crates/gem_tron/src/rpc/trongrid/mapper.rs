use std::str::FromStr;

use super::model::TronGridAccount;
use crate::models::{Transaction, TransactionReceiptData};
use crate::provider::transactions_mapper::map_transaction;
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Chain};

pub struct TronGridMapper;

impl TronGridMapper {
    pub fn get_chain() -> Chain {
        Chain::Tron
    }
    pub fn map_transactions(transactions: Vec<Transaction>, receipts: Vec<TransactionReceiptData>) -> Vec<primitives::Transaction> {
        transactions
            .into_iter()
            .zip(receipts.iter())
            .flat_map(|(transaction, receipt)| map_transaction(Self::get_chain(), transaction, receipt.clone()))
            .collect()
    }

    pub fn map_asset_balances(account: TronGridAccount) -> Vec<AssetBalance> {
        account
            .trc20
            .into_iter()
            .flat_map(|trc20_map| {
                trc20_map.into_iter().map(|(contract_address, balance)| {
                    AssetBalance::new(
                        AssetId::from(Self::get_chain(), Some(contract_address.clone())),
                        BigUint::from_str(balance.as_str()).unwrap_or_default(),
                    )
                })
            })
            .collect()
    }
}
