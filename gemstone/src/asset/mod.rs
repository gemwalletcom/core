use primitives::{Asset, Chain};
use std::str::FromStr;

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

/// Exports functions
#[uniffi::export]
pub fn asset_default_rank(chain: String) -> i32 {
    match Chain::from_str(&chain) {
        Ok(chain) => get_default_rank(chain),
        Err(_) => 10,
    }
}

#[uniffi::export]
pub fn asset_wrapper(chain: String) -> AssetWrapper {
    let chain = Chain::from_str(&chain).unwrap();
    get_asset(chain)
}
