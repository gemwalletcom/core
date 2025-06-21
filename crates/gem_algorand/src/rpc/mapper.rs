use chrono::DateTime;
use primitives::{chain::Chain, AssetBalance, AssetId, AssetType, Transaction, TransactionState, TransactionType};

use crate::rpc::model::{Asset, AssetDetails};

use super::model::{Transaction as AlgoTransaction, TRANSACTION_TYPE_PAY};

pub struct AlgorandMapper;

impl AlgorandMapper {
    pub fn map_transactions(chain: Chain, transactions: Vec<AlgoTransaction>) -> Vec<Transaction> {
        transactions
            .into_iter()
            .flat_map(|transaction| AlgorandMapper::map_transaction(chain, transaction))
            .collect::<Vec<Transaction>>()
    }

    pub fn map_transaction(chain: Chain, transaction: AlgoTransaction) -> Option<Transaction> {
        match transaction.transaction_type.as_str() {
            TRANSACTION_TYPE_PAY => Some(Transaction::new(
                transaction.id.clone(),
                chain.as_asset_id(),
                transaction.sender.clone().unwrap_or_default(),
                transaction.payment_transaction.clone()?.receiver.clone().unwrap_or_default(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                transaction.fee.unwrap_or_default().to_string(),
                chain.as_asset_id(),
                transaction.payment_transaction.clone()?.amount.unwrap_or_default().to_string(),
                transaction.clone().get_memo(),
                None,
                DateTime::from_timestamp(transaction.round_time, 0)?,
            )),
            _ => None,
        }
    }

    pub fn map_assets_balance(assets: Vec<Asset>) -> Vec<AssetBalance> {
        assets
            .into_iter()
            .map(|asset| AssetBalance::new(AssetId::from_token(Chain::Algorand, &asset.asset_id.to_string()), asset.amount.to_string()))
            .collect()
    }

    pub fn map_asset(asset: AssetDetails) -> primitives::Asset {
        primitives::Asset::new(
            AssetId::from_token(Chain::Algorand, &asset.index.to_string()),
            asset.params.name,
            asset.params.unit_name,
            asset.params.decimals as i32,
            AssetType::TOKEN,
        )
    }
}
