pub mod providers;
use async_trait::async_trait;
pub use providers::pyth::provider::PythProvider;

use std::{error::Error, sync::Arc};

use primitives::{AssetId, AssetPrice, Chain};

// impl<T: ChainBlockProvider + ChainTokenDataProvider> ChainProvider for T {}
#[async_trait]
pub trait PriceChainProvider: Send + Sync {
    async fn get_chain_prices(&self, chains: Vec<Chain>) -> Result<Vec<AssetPrice>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait PriceChainAssetsProvider: Send + Sync {
    async fn get_asset_prices(&self, chain: Chain, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl<T: Send + Sync> PriceChainProvider for Arc<T>
where
    T: PriceChainProvider + ?Sized,
{
    async fn get_chain_prices(&self, chains: Vec<primitives::Chain>) -> Result<Vec<primitives::AssetPrice>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_chain_prices(chains).await
    }
}

pub struct PricesDex {}

impl PricesDex {
    pub fn get_chain_prices(&self, _chains: Vec<Chain>) -> Result<Vec<AssetPrice>, Box<dyn Error + Send + Sync>> {
        // Implement the logic to get the chain prices here
        Ok(vec![])
    }
}
