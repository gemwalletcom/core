use std::error::Error;

use primitives::{AssetId, AssetPrice, Chain};

pub trait PriceChainProvider {
    fn get_chain_prices(&self, chains: Vec<Chain>) -> Result<Vec<AssetPrice>, Box<dyn Error + Send + Sync>>;
}

pub trait PriceChainAssetsProvider {
    fn get_asset_prices(&self, chain: Chain, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, Box<dyn Error + Send + Sync>>;
}

pub struct PricesDex {}

impl PricesDex {
    pub fn get_chain_prices(&self, _chains: Vec<Chain>) -> Result<Vec<AssetPrice>, Box<dyn Error + Send + Sync>> {
        // Implement the logic to get the chain prices here
        Ok(vec![])
    }
}
