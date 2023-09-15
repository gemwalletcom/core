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
    pub block_number: i32,
    pub sequence: i32,
    pub fee: String,
    #[serde(rename = "feeAssetId")]
    pub fee_asset_id: AssetId,
    pub value: String,
    pub memo: Option<String>,
    pub direction: TransactionDirection,
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

impl Transaction {

    pub fn id_from(chain: Chain, hash: String) -> String {
        return format!("{}_{}", chain.as_str(), hash);
    }

    pub fn addresses(&self) -> Vec<String> {
        vec![self.from.clone(), self.to.clone()]
    }
}