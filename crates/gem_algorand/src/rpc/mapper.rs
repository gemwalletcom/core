use chrono::DateTime;
use primitives::{chain::Chain, AssetBalance, AssetId, Transaction, TransactionState, TransactionType};

use crate::rpc::model::Asset;

use super::model::{Block, Transaction as AlgoTransaction, TRANSACTION_TYPE_PAY};

pub struct AlgorandMapper;

impl AlgorandMapper {
    pub fn map_transaction(chain: Chain, hash: String, block: Block, transaction: AlgoTransaction) -> Option<Transaction> {
        match transaction.transaction_type.as_str() {
            TRANSACTION_TYPE_PAY => Some(Transaction::new(
                hash,
                chain.as_asset_id(),
                transaction.clone().snd.unwrap_or_default(),
                transaction.clone().rcv.unwrap_or_default(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                transaction.fee.unwrap_or_default().to_string(),
                chain.as_asset_id(),
                transaction.amt.unwrap_or_default().to_string(),
                transaction.clone().get_memo(),
                None,
                DateTime::from_timestamp(block.ts, 0)?,
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
}
