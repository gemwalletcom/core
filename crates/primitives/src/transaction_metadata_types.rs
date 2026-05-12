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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_liquidation: Option<bool>,
    pub provider: Option<PerpetualProvider>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
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
    pub asset_id: NFTAssetId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl TransactionNFTTransferMetadata {
    pub fn new(asset_id: NFTAssetId, name: Option<String>) -> Self {
        Self { asset_id, name }
    }

    pub fn from_asset_id(asset_id: NFTAssetId) -> Self {
        Self { asset_id, name: None }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionSmartContractMetadata {
    pub method_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_transfer_metadata_serialization() {
        let asset_id = NFTAssetId::mock();
        let serialized = asset_id.to_string();
        assert_eq!(
            serde_json::to_value(TransactionNFTTransferMetadata::new(asset_id.clone(), None)).unwrap(),
            serde_json::json!({ "assetId": serialized })
        );
        assert_eq!(
            serde_json::to_value(TransactionNFTTransferMetadata::new(asset_id, Some("NFT".to_string()))).unwrap(),
            serde_json::json!({ "assetId": serialized, "name": "NFT" })
        );
    }
}
