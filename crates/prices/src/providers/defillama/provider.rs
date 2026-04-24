use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset};

use super::client::DefiLlamaClient;
use super::mapper::map_price;

const COINS_PER_REQUEST: usize = 100;

pub struct DefiLlamaProvider {
    client: DefiLlamaClient,
}

impl DefiLlamaProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: DefiLlamaClient::new(client),
        }
    }
}

#[async_trait]
impl PriceAssetsProvider for DefiLlamaProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::DefiLlama
    }

    async fn get_assets(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::with_capacity(mappings.len());
        for chunk in mappings.chunks(COINS_PER_REQUEST) {
            let coins: Vec<String> = chunk.iter().map(|m| m.provider_price_id.clone()).collect();
            let response = self.client.get_prices(&coins).await?;
            for mapping in chunk {
                if let Some(coin) = response.coins.get(&mapping.provider_price_id) {
                    results.push(map_price(mapping.clone(), coin));
                }
            }
        }
        Ok(results)
    }
}
