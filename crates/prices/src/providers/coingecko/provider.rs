use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use coingecko::{COINGECKO_CHAIN_MAP, CoinGeckoClient, get_chain_for_coingecko_platform_id};
use primitives::AssetId;

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider};

use super::mapper::map_coin_market;

const MAX_MARKETS_PER_PAGE: usize = 250;

pub struct CoinGeckoPricesProvider {
    client: CoinGeckoClient,
}

impl CoinGeckoPricesProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: CoinGeckoClient::new(api_key),
        }
    }

    pub fn from_client(client: CoinGeckoClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl PriceAssetsProvider for CoinGeckoPricesProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Coingecko
    }

    async fn get_assets(&self) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        let coins = self.client.get_coin_list().await?;
        let mut mappings = Vec::with_capacity(coins.len());

        for coin in coins {
            if let Some(chain) = COINGECKO_CHAIN_MAP.get(coin.id.as_str()) {
                mappings.push(AssetPriceMapping::new(chain.as_asset_id(), coin.id.clone()));
            }
            for (platform_id, contract) in &coin.platforms {
                let Some(chain) = get_chain_for_coingecko_platform_id(platform_id) else {
                    continue;
                };
                let Some(token_id) = contract.as_ref().filter(|s| !s.is_empty()) else {
                    continue;
                };
                mappings.push(AssetPriceMapping::new(AssetId::from(chain, Some(token_id.clone())), coin.id.clone()));
            }
        }

        Ok(mappings)
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }

        let mut by_id: HashMap<String, AssetPriceMapping> = HashMap::new();
        for mapping in mappings {
            by_id.insert(mapping.provider_price_id.clone(), mapping);
        }

        let mut out = Vec::with_capacity(by_id.len());
        let ids: Vec<String> = by_id.keys().cloned().collect();
        for chunk in ids.chunks(MAX_MARKETS_PER_PAGE) {
            let coin_markets = self.client.get_coin_markets_ids(chunk.to_vec(), MAX_MARKETS_PER_PAGE).await?;
            for market in coin_markets {
                if let Some(mapping) = by_id.get(&market.id).cloned() {
                    out.push(map_coin_market(market, mapping));
                }
            }
        }
        Ok(out)
    }
}
