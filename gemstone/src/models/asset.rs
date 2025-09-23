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

pub type AssetWrapper = GemAsset;

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
    fn from(asset: GemAsset) -> Self {
        let id = AssetId::new(&asset.id).unwrap();
        let asset_type = AssetType::from_str(&asset.asset_type).unwrap_or(AssetType::NATIVE);
        Asset {
            id: id.clone(),
            chain: id.chain,
            token_id: id.token_id,
            name: asset.name,
            symbol: asset.symbol,
            decimals: asset.decimals,
            asset_type,
        }
    }
}

pub fn get_default_rank(chain: Chain) -> i32 {
    chain.rank()
}

pub fn get_asset(chain: Chain) -> GemAsset {
    let asset = Asset::from_chain(chain);
    GemAsset {
        id: asset.id.to_string(),
        name: asset.name,
        symbol: asset.symbol,
        decimals: asset.decimals,
        asset_type: asset.asset_type.as_ref().to_string(),
    }
}

#[uniffi::export]
pub fn asset_default_rank(chain: String) -> i32 {
    match Chain::from_str(&chain) {
        Ok(chain) => get_default_rank(chain),
        Err(_) => 10,
    }
}

#[uniffi::export]
pub fn asset_wrapper(chain: String) -> GemAsset {
    let chain = Chain::from_str(&chain).unwrap();
    get_asset(chain)
}