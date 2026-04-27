pub mod model;
pub mod providers;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{AssetId, ChartValue};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

pub use model::{AssetPriceFull, AssetPriceMapping, PriceProviderAsset, PriceProviderAssetMetadata};
pub use primitives::PriceProvider;
pub use providers::coingecko::provider::CoinGeckoPricesProvider;
pub use providers::defillama::provider::DefiLlamaProvider;
pub use providers::pyth::provider::PythProvider;

#[derive(Clone, Copy, Debug, Default)]
pub struct PriceProviderConfig {
    pub min_score: f64,
}

pub use providers::jupiter::provider::JupiterProvider;

#[derive(Clone, Debug)]
pub struct PriceProviderEndpoints {
    pub coingecko_api_key: String,
    pub pyth_url: String,
    pub jupiter_url: String,
    pub defillama_url: String,
}

impl PriceProviderEndpoints {
    pub fn provider(&self, provider: PriceProvider, config: PriceProviderConfig) -> Arc<dyn PriceAssetsProvider> {
        match provider {
            PriceProvider::Coingecko => Arc::new(CoinGeckoPricesProvider::new(&self.coingecko_api_key, config)),
            PriceProvider::Pyth => Arc::new(PythProvider::new(ReqwestClient::new(self.pyth_url.clone(), reqwest::Client::new()), config)),
            PriceProvider::Jupiter => Arc::new(JupiterProvider::new(ReqwestClient::new(self.jupiter_url.clone(), reqwest::Client::new()), config)),
            PriceProvider::DefiLlama => Arc::new(DefiLlamaProvider::new(ReqwestClient::new(self.defillama_url.clone(), reqwest::Client::new()))),
        }
    }
}

#[async_trait]
pub trait PriceAssetsProvider: Send + Sync {
    fn provider(&self) -> PriceProvider;
    async fn get_assets(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>>;
    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>>;
    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>>;
    async fn get_assets_new(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
    async fn get_assets_metadata(&self, _mappings: Vec<AssetPriceMapping>) -> Result<Vec<PriceProviderAssetMetadata>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>>;
    async fn get_charts_daily(&self, _provider_price_id: &str) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
    async fn get_charts_hourly(&self, _provider_price_id: &str, _duration: Duration) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
