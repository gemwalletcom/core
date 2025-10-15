use std::error::Error;

use chrono::Utc;
use primitives::Chain;

use crate::{AssetPriceFeed, DexAssetPrice, PriceChainAssetsProvider, PriceFeedId, PriceFeedProvider};
use async_trait::async_trait;

use super::{
    client::PythClient,
    mapper::{asset_id_for_feed_id, price_feed_id_for_chain},
};

pub struct PythProvider {
    pub pyth_client: PythClient,
}

#[async_trait]
impl PriceChainAssetsProvider for PythProvider {
    fn get_provider(&self) -> PriceFeedProvider {
        PriceFeedProvider::Pyth
    }

    async fn get_supported_feeds(&self) -> Result<Vec<AssetPriceFeed>, Box<dyn Error + Send + Sync>> {
        let feeds = self.pyth_client.get_price_feeds().await?;
        Ok(feeds
            .into_iter()
            .filter_map(|feed| {
                asset_id_for_feed_id(&feed.id).map(|asset_id| AssetPriceFeed::new(PriceFeedId::new(PriceFeedProvider::Pyth, feed.id.clone()), asset_id))
            })
            .collect())
    }

    async fn get_assets_prices(&self, feed_ids: Vec<PriceFeedId>) -> Result<Vec<DexAssetPrice>, Box<dyn Error + Send + Sync>> {
        use std::collections::{HashMap, HashSet};

        let feed_id_set: HashSet<String> = feed_ids.iter().map(|f| f.feed_id.clone()).collect();

        let chains = Chain::all();
        let chain_feed_map: HashMap<String, Vec<Chain>> = chains.into_iter().fold(HashMap::new(), |mut acc, chain| {
            let feed_id = price_feed_id_for_chain(chain).to_string();
            if feed_id_set.contains(&feed_id) {
                acc.entry(feed_id).or_default().push(chain);
            }
            acc
        });

        let unique_ids: Vec<String> = chain_feed_map.keys().cloned().collect();
        if unique_ids.is_empty() {
            return Ok(vec![]);
        }

        let prices = self.pyth_client.get_asset_prices(unique_ids.clone()).await?;
        let updated_at = Utc::now();

        let price_map: HashMap<String, f64> = unique_ids.into_iter().zip(prices).map(|(id, p)| (id, p.price)).collect();

        Ok(chain_feed_map
            .into_iter()
            .flat_map(|(feed_id, chains)| {
                let price = price_map.get(&feed_id).copied().unwrap_or(0.0);
                chains.into_iter().map(move |chain| {
                    let asset_price_feed = AssetPriceFeed::new(PriceFeedId::new(PriceFeedProvider::Pyth, feed_id.clone()), chain.as_asset_id());
                    DexAssetPrice::new(chain.as_asset_id(), asset_price_feed, price, updated_at)
                })
            })
            .collect())
    }
}

#[cfg(all(test, feature = "price_integration_tests"))]
mod tests {
    use super::super::testkit::create_pyth_test_provider;
    use crate::{PriceChainAssetsProvider, PriceFeedProvider};
    use primitives::Chain;

    #[tokio::test]
    async fn test_pyth_get_provider() {
        let provider = create_pyth_test_provider();
        assert_eq!(provider.get_provider(), PriceFeedProvider::Pyth);
    }

    #[tokio::test]
    async fn test_pyth_get_supported_feeds() {
        let provider = create_pyth_test_provider();
        let result = provider.get_supported_feeds().await;

        assert!(result.is_ok());
        let supported = result.unwrap();
        assert!(!supported.is_empty());

        for feed in &supported {
            assert_eq!(feed.price_feed_id.provider, PriceFeedProvider::Pyth);
            assert!(!feed.price_feed_id.feed_id.is_empty());
            assert!(feed.get_id().starts_with("pyth_"));
        }
    }

    #[tokio::test]
    async fn test_pyth_get_assets_prices() {
        use super::mapper::price_feed_id_for_chain;

        let provider = create_pyth_test_provider();
        let feed_ids = Chain::all()
            .iter()
            .map(|chain| PriceFeedId::new(PriceFeedProvider::Pyth, price_feed_id_for_chain(*chain).to_string()))
            .collect();
        let result = provider.get_assets_prices(feed_ids).await;

        assert!(result.is_ok());
        let prices = result.unwrap();
        assert!(!prices.is_empty());
        assert!(prices.len() == Chain::all().len());
    }
}
