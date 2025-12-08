use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, NFTAssetId, PerpetualDirection, PerpetualProvider, stake_type::Resource};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionPerpetualMetadata {
    pub pnl: f64,
    pub price: f64,
    pub direction: PerpetualDirection,
    pub provider: Option<PerpetualProvider>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionResourceTypeMetadata {
    pub resource_type: Resource,
}

impl TransactionResourceTypeMetadata {
    pub fn new(resource_type: Resource) -> Self {
        Self { resource_type }
    }
}
