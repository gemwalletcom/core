pub mod model;
pub mod providers;

use async_trait::async_trait;
use std::error::Error;

pub use model::{AssetPriceFeed, DexAssetPrice};
pub use primitives::{PriceFeedId, PriceFeedProvider};
pub use providers::jupiter::provider::JupiterProvider;
pub use providers::pyth::provider::PythProvider;

#[async_trait]
pub trait PriceChainAssetsProvider: Send + Sync {
    fn get_provider(&self) -> PriceFeedProvider;
    async fn get_supported_feeds(&self) -> Result<Vec<AssetPriceFeed>, Box<dyn Error + Send + Sync>>;
    async fn get_assets_prices(&self, feed_ids: Vec<PriceFeedId>) -> Result<Vec<DexAssetPrice>, Box<dyn Error + Send + Sync>>;
}
