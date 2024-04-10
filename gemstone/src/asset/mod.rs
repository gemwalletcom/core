use primitives::{Asset, Chain};

pub fn get_default_rank(chain: Chain) -> i32 {
    chain.rank()
}

#[derive(uniffi::Record, Clone)]
pub struct AssetWrapper {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub asset_type: String,
}

pub fn get_asset(chain: Chain) -> AssetWrapper {
    let asset = Asset::from_chain(chain);
    AssetWrapper {
        id: asset.id.to_string(),
        name: asset.name,
        symbol: asset.symbol,
        decimals: asset.decimals,
        asset_type: asset.asset_type.as_ref().to_string(),
    }
}
