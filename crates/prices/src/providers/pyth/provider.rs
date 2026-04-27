use std::collections::{HashMap, HashSet};
use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::AssetId;

use super::{
    client::PythClient,
    mapper::{asset_ids_for_feed_id, price_feed_id_for_chain},
};
use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset, PriceProviderConfig};

pub struct PythProvider {
    pyth_client: PythClient,
}

impl PythProvider {
    pub fn new(client: ReqwestClient, _config: PriceProviderConfig) -> Self {
        Self {
            pyth_client: PythClient::new(client),
        }
    }
}

#[async_trait]
impl PriceAssetsProvider for PythProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Pyth
    }

    async fn get_assets(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let feeds = self.pyth_client.get_price_feeds().await?;
        Ok(feeds
            .into_iter()
            .flat_map(|feed| {
                asset_ids_for_feed_id(&feed.id)
                    .into_iter()
                    .map(move |asset_id| AssetPriceMapping::new(asset_id, feed.id.clone()))
            })
            .map(|m| PriceProviderAsset::new(m, None))
            .collect())
    }

    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(asset_id
            .is_native()
            .then(|| AssetPriceMapping::new(asset_id.clone(), price_feed_id_for_chain(asset_id.chain).to_string()))
            .into_iter()
            .collect())
    }

    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(asset_ids_for_feed_id(provider_price_id)
            .into_iter()
            .map(|asset_id| AssetPriceMapping::new(asset_id, provider_price_id.to_string()))
            .collect())
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        let feed_ids: Vec<String> = mappings
            .iter()
            .map(|mapping| mapping.provider_price_id.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if feed_ids.is_empty() {
            return Ok(vec![]);
        }

        let prices = self.pyth_client.get_asset_prices(feed_ids.clone()).await?;
        let prices_by_feed_id: HashMap<String, f64> = feed_ids.into_iter().zip(prices).map(|(id, price)| (id, price.price)).collect();

        Ok(mappings
            .into_iter()
            .filter_map(|mapping| {
                prices_by_feed_id
                    .get(&mapping.provider_price_id)
                    .map(|price| AssetPriceFull::simple(mapping, *price, 0.0, PriceProvider::Pyth))
            })
            .collect())
    }
}

#[cfg(all(test, feature = "price_integration_tests"))]
mod tests {
    use super::super::mapper::price_feed_id_for_chain;
    use super::super::testkit::create_pyth_test_provider;
    use crate::{AssetPriceMapping, PriceAssetsProvider, PriceProvider};
    use primitives::Chain;

    #[tokio::test]
    async fn test_pyth_provider_basic() {
        let provider = create_pyth_test_provider();
        assert_eq!(provider.provider(), PriceProvider::Pyth);

        let supported = provider.get_assets().await.unwrap();
        assert!(!supported.is_empty());
        for asset in &supported {
            assert!(!asset.mapping.provider_price_id.is_empty());
        }

        let mappings: Vec<AssetPriceMapping> = Chain::all()
            .iter()
            .map(|chain| AssetPriceMapping::new(chain.as_asset_id(), price_feed_id_for_chain(*chain).to_string()))
            .collect();
        let prices = provider.get_prices(mappings).await.unwrap();
        assert!(!prices.is_empty());
        assert_eq!(prices.len(), Chain::all().len());
    }
}
