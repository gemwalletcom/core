use std::error::Error;

use gem_client::ReqwestClient;
use primitives::Chain;

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset};
use async_trait::async_trait;

use super::{
    client::PythClient,
    mapper::{asset_id_for_feed_id, price_feed_id_for_chain},
};

pub struct PythProvider {
    pyth_client: PythClient,
}

impl PythProvider {
    pub fn new(client: ReqwestClient) -> Self {
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
            .filter_map(|feed| asset_id_for_feed_id(&feed.id).map(|asset_id| AssetPriceMapping::new(asset_id, feed.id)))
            .map(|m| PriceProviderAsset::new(m, None))
            .collect())
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        use std::collections::{HashMap, HashSet};

        let feed_id_set: HashSet<String> = mappings.iter().map(|m| m.provider_price_id.clone()).collect();

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
        let price_map: HashMap<String, f64> = unique_ids.into_iter().zip(prices).map(|(id, p)| (id, p.price)).collect();

        Ok(chain_feed_map
            .into_iter()
            .flat_map(|(feed_id, chains)| {
                let price = price_map.get(&feed_id).copied().unwrap_or(0.0);
                chains.into_iter().map(move |chain| {
                    let mapping = AssetPriceMapping::new(chain.as_asset_id(), feed_id.clone());
                    AssetPriceFull::simple(mapping, price, 0.0, PriceProvider::Pyth)
                })
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
