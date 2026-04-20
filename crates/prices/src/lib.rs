pub mod model;
pub mod providers;

use async_trait::async_trait;
use std::error::Error;

pub use model::{AssetPriceFull, AssetPriceMapping};
pub use primitives::PriceProvider;
pub use providers::coingecko::provider::CoinGeckoPricesProvider;
pub use providers::jupiter::provider::JupiterProvider;
pub use providers::pyth::provider::PythProvider;

#[async_trait]
pub trait PriceAssetsProvider: Send + Sync {
    fn provider(&self) -> PriceProvider;
    async fn get_assets(&self) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>>;
    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>>;
}
