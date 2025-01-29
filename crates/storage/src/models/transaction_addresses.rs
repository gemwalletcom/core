use diesel::prelude::*;
use primitives::TransactionSwapMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionAddresses {
    pub chain_id: String,
    pub asset_id: String,
    pub transaction_id: String,
    pub address: String,
}

impl TransactionAddresses {
    pub fn from_primitive(transaction: primitives::Transaction) -> Vec<TransactionAddresses> {
        let transaction_id = transaction.clone().id;
        match transaction.transaction_type {
            primitives::TransactionType::Transfer
            | primitives::TransactionType::TokenApproval
            | primitives::TransactionType::StakeDelegate
            | primitives::TransactionType::StakeUndelegate
            | primitives::TransactionType::StakeRewards
            | primitives::TransactionType::StakeRedelegate
            | primitives::TransactionType::StakeWithdraw
            | primitives::TransactionType::TransferNFT => transaction
                .addresses()
                .into_iter()
                .map(|x| Self {
                    chain_id: transaction.asset_id.chain.as_ref().to_string(),
                    asset_id: transaction.asset_id.to_string(),
                    transaction_id: transaction_id.clone(),
                    address: x,
                })
                .collect(),
            primitives::TransactionType::Swap => {
                let metadata: TransactionSwapMetadata = serde_json::from_value(transaction.metadata.clone().unwrap()).unwrap();
                let from_asset = metadata.from_asset.clone();
                let to_asset = metadata.to_asset.clone();
                vec![
                    Self {
                        chain_id: from_asset.chain.as_ref().to_string(),
                        asset_id: from_asset.to_string(),
                        transaction_id: transaction_id.clone(),
                        address: transaction.from.clone(),
                    },
                    Self {
                        chain_id: to_asset.chain.as_ref().to_string(),
                        asset_id: to_asset.clone().to_string(),
                        transaction_id: transaction_id.clone(),
                        address: transaction.clone().to,
                    },
                ]
            }
            primitives::TransactionType::AssetActivation => todo!(),
        }
    }
}

#[derive(Queryable, Debug, Clone)]
pub struct AddressChainIdResult {
    pub address: String,
    pub chain_id: String,
}
