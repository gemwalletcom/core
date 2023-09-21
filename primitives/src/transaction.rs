use typeshare::typeshare;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

use crate::{asset_id::AssetId, transaction_type::TransactionType, transaction_state::TransactionState, transaction_direction::TransactionDirection, Chain};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct Transaction {
    pub id: String,
    pub hash: String,
    #[serde(rename = "assetId")]
    pub asset_id: AssetId,
    pub from: String,
    pub to: String,
    pub contract: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub state: TransactionState,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    pub sequence: String,
    pub fee: String,
    #[serde(rename = "feeAssetId")]
    pub fee_asset_id: AssetId,
    pub value: String,
    pub memo: Option<String>,
    pub direction: TransactionDirection,
    pub block_created_at: NaiveDateTime,
}

impl Transaction {

    pub fn new(
        hash: String,
        asset_id: AssetId,
        from: String,
        to: String,
        contract: Option<String>,
        transaction_type: TransactionType,
        state: TransactionState,
        block_number: String,
        sequence: String,
        fee: String,
        fee_asset_id: AssetId,
        value: String,
        memo: Option<String>,
        direction: TransactionDirection,
        block_created_at: NaiveDateTime,
    ) -> Self {
        let id = Self::id_from(asset_id.clone().chain, hash.clone());
        Self {
            id,
            hash,
            asset_id,
            from,
            to,
            contract,
            transaction_type,
            state,
            block_number,
            sequence,
            fee,
            fee_asset_id,
            value,
            memo,
            direction,
            block_created_at,
        }
    }

    pub fn id_from(chain: Chain, hash: String) -> String {
        format!("{}_{}", chain.as_str(), hash)
    }

    pub fn addresses(&self) -> Vec<String> {
        vec![self.from.clone(), self.to.clone()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TransactionsFetchOption  {
    pub asset_id: Option<String>,
    pub from_timestamp: Option<u32>
}