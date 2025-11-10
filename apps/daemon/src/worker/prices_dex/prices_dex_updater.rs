use gem_tracing::info_with_fields;
use prices_dex::providers::pyth::client::PythClient;
use prices_dex::{AssetPriceFeed, DexAssetPrice, JupiterProvider, PriceChainAssetsProvider, PriceFeedProvider, PythProvider};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use storage::Database;
use storage::models::{PriceDex, PriceDexAsset};

pub struct PricesDexUpdater {
    provider_type: PriceFeedProvider,
    provider: Arc<dyn PriceChainAssetsProvider>,
    database: Database,
}

impl PricesDexUpdater {
    pub fn new(provider_type: PriceFeedProvider, url: &str, database: Database) -> Self {
        let provider: Arc<dyn PriceChainAssetsProvider> = match provider_type {
            PriceFeedProvider::Pyth => Arc::new(PythProvider {
                pyth_client: PythClient::new(url),
            }),
            PriceFeedProvider::Jupiter => Arc::new(JupiterProvider::new(url)),
        };

        Self {
            provider_type,
            provider,
            database,
        }
    }

    pub async fn update_feeds(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let feeds = self.provider.get_supported_feeds().await?;
        self.save_feeds(feeds)
    }

    pub async fn update_prices(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let feed_ids = self.database.client()?.prices_dex().get_feed_ids_by_provider(self.provider_type.clone())?;
        let prices = self.provider.get_assets_prices(feed_ids).await?;
        self.save_prices(prices)
    }

    fn validate_asset_ids(&mut self, asset_ids: Vec<String>) -> Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>> {
        let existing_assets = self.database.client()?.assets().get_assets_basic(asset_ids)?;
        Ok(existing_assets.into_iter().map(|a| a.asset.id.to_string()).collect())
    }

    fn save_feeds(&mut self, feeds: Vec<AssetPriceFeed>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let asset_ids: Vec<String> = feeds.iter().map(|feed| feed.asset_id.to_string()).collect();
        let existing_asset_ids = self.validate_asset_ids(asset_ids)?;

        let valid_feeds: Vec<&AssetPriceFeed> = feeds.iter().filter(|feed| existing_asset_ids.contains(&feed.asset_id.to_string())).collect();
        let missing_feeds: Vec<&AssetPriceFeed> = feeds.iter().filter(|feed| !existing_asset_ids.contains(&feed.asset_id.to_string())).collect();

        if valid_feeds.is_empty() {
            return Ok(());
        }

        let feed_records: Vec<PriceDex> = valid_feeds
            .iter()
            .map(|feed| PriceDex::new(feed.get_id(), self.provider_type.as_ref().to_string(), 0.0, chrono::Utc::now().naive_utc()))
            .collect();

        self.database.client()?.prices_dex().add_prices_dex(feed_records)?;

        let asset_records: Vec<PriceDexAsset> = valid_feeds
            .iter()
            .map(|feed| PriceDexAsset::new(feed.asset_id.to_string(), feed.get_id()))
            .collect();

        self.database.client()?.prices_dex().set_prices_dex_assets(asset_records)?;

        info_with_fields!(
            "save_feeds",
            provider = self.provider_type.as_ref(),
            feeds = valid_feeds.len(),
            missing_feeds = missing_feeds.len(),
            total = feeds.len()
        );

        Ok(())
    }

    fn save_prices(&mut self, prices: Vec<DexAssetPrice>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let asset_ids: Vec<String> = prices.iter().map(|p| p.asset_id.to_string()).collect();
        let existing_asset_ids = self.validate_asset_ids(asset_ids)?;

        let valid_prices: Vec<&DexAssetPrice> = prices.iter().filter(|p| existing_asset_ids.contains(&p.asset_id.to_string())).collect();

        if valid_prices.is_empty() {
            return Ok(0);
        }

        let mut feed_map: HashMap<String, &DexAssetPrice> = HashMap::new();
        for price in &valid_prices {
            feed_map.insert(price.price_feed.get_id(), *price);
        }

        let values: Vec<PriceDex> = feed_map
            .values()
            .map(|p| {
                PriceDex::new(
                    p.price_feed.get_id(),
                    self.provider_type.as_ref().to_string(),
                    p.price,
                    p.updated_at.naive_utc(),
                )
            })
            .collect();

        self.database.client()?.prices_dex().set_prices_dex(values.clone())?;

        Ok(values.len())
    }
}
