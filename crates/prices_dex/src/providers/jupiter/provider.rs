use std::error::Error;

use async_trait::async_trait;
use chrono::Utc;

use crate::{AssetPriceFeed, DexAssetPrice, PriceChainAssetsProvider, PriceFeedId, PriceFeedProvider};

use super::{client::JupiterClient, mapper::map_id_to_asset_id};

pub struct JupiterProvider {
    pub jupiter_client: JupiterClient,
}

impl JupiterProvider {
    pub fn new(base_url: &str) -> Self {
        Self {
            jupiter_client: JupiterClient::new(base_url),
        }
    }
}

#[async_trait]
impl PriceChainAssetsProvider for JupiterProvider {
    fn get_provider(&self) -> PriceFeedProvider {
        PriceFeedProvider::Jupiter
    }

    async fn get_supported_feeds(&self) -> Result<Vec<AssetPriceFeed>, Box<dyn Error + Send + Sync>> {
        let tokens = self.jupiter_client.get_verified_tokens().await?;

        let feeds = tokens
            .into_iter()
            .take(10)
            .map(|token| {
                let asset_id = map_id_to_asset_id(&token);
                AssetPriceFeed::new(PriceFeedId::new(PriceFeedProvider::Jupiter, token), asset_id)
            })
            .collect();

        Ok(feeds)
    }

    async fn get_assets_prices(&self, feed_ids: Vec<PriceFeedId>) -> Result<Vec<DexAssetPrice>, Box<dyn Error + Send + Sync>> {
        use std::collections::HashSet;

        if feed_ids.is_empty() {
            return Ok(vec![]);
        }

        let feed_id_set: HashSet<String> = feed_ids.iter().map(|f| f.feed_id.clone()).collect();
        let all_feeds = self.get_supported_feeds().await?;
        let feeds: Vec<AssetPriceFeed> = all_feeds.into_iter().filter(|feed| feed_id_set.contains(&feed.price_feed_id.feed_id)).collect();

        if feeds.is_empty() {
            return Ok(vec![]);
        }

        let ids = feeds.iter().map(|feed| feed.price_feed_id.feed_id.clone()).collect();
        let prices = self.jupiter_client.get_asset_prices(ids).await?;
        let updated_at = Utc::now();

        Ok(feeds
            .into_iter()
            .zip(prices)
            .map(|(feed, price)| DexAssetPrice::new(feed.asset_id.clone(), feed, price.price, updated_at))
            .collect())
    }
}

#[cfg(all(test, feature = "price_integration_tests"))]
mod tests {
    use super::super::testkit::create_jupiter_test_provider;
    use crate::{PriceChainAssetsProvider, PriceFeedProvider};

    #[tokio::test]
    async fn test_jupiter_get_provider() {
        let provider = create_jupiter_test_provider();
        assert_eq!(provider.get_provider(), PriceFeedProvider::Jupiter);
    }

    #[tokio::test]
    async fn test_jupiter_get_supported_feeds() {
        let provider = create_jupiter_test_provider();
        let result = provider.get_supported_feeds().await;

        assert!(result.is_ok());
        let supported = result.unwrap();
        assert!(supported.is_empty());
    }
}
