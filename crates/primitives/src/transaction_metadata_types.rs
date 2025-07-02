use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, NFTAssetId};

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
}

impl TransactionNFTTransferMetadata {
    pub fn new(asset_id: String) -> Self {
        Self { asset_id }
    }

    pub fn from_asset_id(asset_id: NFTAssetId) -> Self {
        Self {
            asset_id: asset_id.to_string(),
        }
    }
}
