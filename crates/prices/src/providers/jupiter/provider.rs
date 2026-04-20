use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use chrono::Utc;
use gem_client::ReqwestClient;
use primitives::{AssetMarket, Price};

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider};

use super::client::JupiterClient;
use super::mapper::{to_asset_price_mapping, to_jupiter_token_id};
use super::model::VerifiedToken;

const MIN_ORGANIC_SCORE: f64 = 50.0;

pub struct JupiterProvider {
    jupiter_client: JupiterClient,
}

impl JupiterProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            jupiter_client: JupiterClient::new(client),
        }
    }

    async fn verified_tokens(&self) -> Result<Vec<VerifiedToken>, Box<dyn Error + Send + Sync>> {
        Ok(self.jupiter_client.get_verified_tokens().await?.into_iter().filter(|t| t.organic_score >= MIN_ORGANIC_SCORE).collect())
    }
}

#[async_trait]
impl PriceAssetsProvider for JupiterProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Jupiter
    }

    async fn get_assets(&self) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(self.verified_tokens().await?.into_iter().map(|t| to_asset_price_mapping(&t.id)).collect())
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
    let market = AssetMarket {
        market_cap: token.mcap,
        market_cap_fdv: token.fdv,
        circulating_supply: token.circ_supply,
        total_supply: token.total_supply,
        ..AssetMarket::default()
    };
    AssetPriceFull::new(mapping, Price::new(token.usd_price, token.stats24h.price_change, Utc::now(), PriceProvider::Jupiter), Some(market))
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
        for mapping in &supported {
            assert!(!mapping.provider_price_id.is_empty());
        }
    }
}
