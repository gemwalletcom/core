use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::AssetId;

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset, PriceProviderConfig};

use super::client::JupiterClient;
use super::mapper::{to_asset_price_mapping, to_jupiter_token_id};
use super::model::VerifiedToken;

pub struct JupiterProvider {
    jupiter_client: JupiterClient,
    config: PriceProviderConfig,
}

impl JupiterProvider {
    pub fn new(client: ReqwestClient, config: PriceProviderConfig) -> Self {
        Self {
            jupiter_client: JupiterClient::new(client),
            config,
        }
    }

    async fn verified_tokens(&self) -> Result<Vec<VerifiedToken>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .jupiter_client
            .get_verified_tokens()
            .await?
            .into_iter()
            .filter(|t| t.organic_score >= self.config.min_score)
            .collect())
    }
}

#[async_trait]
impl PriceAssetsProvider for JupiterProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Jupiter
    }

    async fn get_assets(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .verified_tokens()
            .await?
            .into_iter()
            .map(|t| PriceProviderAsset::with_price(to_asset_price_mapping(&t.id), None, Some(t.usd_price), Some(t.stats24h.price_change)))
            .collect())
    }

    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(asset_id
            .token_id
            .clone()
            .map(|token_id| AssetPriceMapping::new(asset_id.clone(), token_id))
            .into_iter()
            .collect())
    }

    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(vec![to_asset_price_mapping(provider_price_id)])
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }
        let tokens: HashMap<String, VerifiedToken> = self.verified_tokens().await?.into_iter().map(|t| (t.id.clone(), t)).collect();
        Ok(mappings
            .into_iter()
            .filter_map(|mapping| {
                tokens
                    .get(&to_jupiter_token_id(&mapping.provider_price_id))
                    .map(|token| to_asset_price_full(mapping, token))
            })
            .collect())
    }
}

fn to_asset_price_full(mapping: AssetPriceMapping, token: &VerifiedToken) -> AssetPriceFull {
    AssetPriceFull::simple(mapping, token.usd_price, token.stats24h.price_change, PriceProvider::Jupiter)
}

#[cfg(all(test, feature = "price_integration_tests"))]
mod tests {
    use super::super::testkit::create_jupiter_test_provider;
    use crate::{PriceAssetsProvider, PriceProvider};

    #[tokio::test]
    async fn test_jupiter_provider_basic() {
        let provider = create_jupiter_test_provider();
        assert_eq!(provider.provider(), PriceProvider::Jupiter);

        let supported = provider.get_assets().await.unwrap();
        assert!(!supported.is_empty());
        for asset in &supported {
            assert!(!asset.mapping.provider_price_id.is_empty());
        }
    }
}
