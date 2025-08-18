use primitives::{Asset, AssetId, AssetType, Chain};
use std::str::FromStr;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAsset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub asset_type: String,
}

impl From<Asset> for GemAsset {
    fn from(asset: Asset) -> Self {
        Self {
            id: asset.id.to_string(),
            name: asset.name,
            symbol: asset.symbol,
            decimals: asset.decimals,
            asset_type: asset.asset_type.as_ref().to_string(),
        }
    }
}

impl From<GemAsset> for Asset {
    fn from(gem_asset: GemAsset) -> Self {
        // Parse the AssetId from string
        let id = AssetId::new(&gem_asset.id).unwrap_or_else(|| {
            // Fallback: assume it's just a chain
            let chain = Chain::from_str(&gem_asset.id).unwrap_or(Chain::Bitcoin);
            AssetId::from_chain(chain)
        });
        
        let asset_type = AssetType::from_str(&gem_asset.asset_type).unwrap_or(AssetType::NATIVE);
        
        Asset {
            id: id.clone(),
            chain: id.chain,
            token_id: id.token_id,
            name: gem_asset.name,
            symbol: gem_asset.symbol,
            decimals: gem_asset.decimals,
            asset_type,
        }
    }
}
