use crate::rpc::{
    model::{Transaction, TransactionReceiptData},
    TronMapper,
};

use super::model::TronGridAccount;
use primitives::{AssetBalance, AssetId, Chain};

pub struct TronGridMapper;

impl TronGridMapper {
    pub fn get_chain() -> Chain {
        Chain::Tron
    }
    pub fn map_transactions(transactions: Vec<Transaction>, reciepts: Vec<TransactionReceiptData>) -> Vec<primitives::Transaction> {
        transactions
            .into_iter()
            .zip(reciepts.iter())
            .flat_map(|(transaction, receipt)| TronMapper::map_transaction(Self::get_chain(), transaction, receipt.clone()))
            .collect()
    }

    pub fn map_asset_balances(account: TronGridAccount) -> Vec<AssetBalance> {
        account
            .trc20
            .into_iter()
            .flat_map(|trc20_map| {
                trc20_map.into_iter().map(|(contract_address, balance)| AssetBalance {
                    asset_id: AssetId::from(Self::get_chain(), Some(contract_address.clone())),
                    balance: balance.to_string(),
                })
            })
            .collect()
    }
}
