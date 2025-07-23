use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, NFTAssetId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub enum StakeType {
    Stake,
    Unstake,
    Redelegate,
    Rewards,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionStakeMetadata {
    pub stake_type: StakeType,
    pub asset_id: AssetId,
    pub validator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_validator: Option<String>,
    pub delegator: Option<String>,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionSwapMetadata {
    pub from_asset: AssetId,
    pub from_value: String,
    pub to_asset: AssetId,
    pub to_value: String,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionNFTTransferMetadata {
    pub asset_id: String,
    pub name: Option<String>,
}

impl TransactionNFTTransferMetadata {
    pub fn new(asset_id: String, name: Option<String>) -> Self {
        Self { asset_id, name }
    }

    pub fn from_asset_id(asset_id: NFTAssetId) -> Self {
        Self {
            asset_id: asset_id.to_string(),
            name: None,
        }
    }
}
