use std::collections::{HashMap, HashSet};
use std::error::Error;

use async_trait::async_trait;
use coingecko::CoinGeckoClient;
use primitives::ChartValue;

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset};

use super::mapper::{map_coin_markets, map_coins_to_assets, map_coins_to_mappings, map_market_chart_daily};

const MAX_MARKETS_PER_PAGE: usize = 250;
const MAX_RANKED_PAGES: usize = 20;

pub struct CoinGeckoPricesProvider {
    client: CoinGeckoClient,
}

impl CoinGeckoPricesProvider {
    pub fn new(api_key: &str) -> Self {
        Self::from_client(CoinGeckoClient::new(api_key))
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

    async fn get_assets(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let markets = self.client.get_all_coin_markets(None, MAX_MARKETS_PER_PAGE, MAX_RANKED_PAGES).await?;
        let markets_by_id: HashMap<String, _> = markets.into_iter().map(|m| (m.id.clone(), m)).collect();
        let coins = self.client.get_coin_list().await?.into_iter().filter(|c| markets_by_id.contains_key(&c.id)).collect();
        Ok(map_coins_to_assets(coins, markets_by_id))
    }

    async fn get_assets_new(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let trending: HashSet<String> = self.client.get_search_trending().await?.get_coins_ids().into_iter().collect();
        if trending.is_empty() {
            return Ok(vec![]);
        }
        let coins = self.client.get_coin_list().await?.into_iter().filter(|c| trending.contains(&c.id)).collect();
        Ok(map_coins_to_mappings(coins).into_iter().map(|m| PriceProviderAsset::new(m, None)).collect())
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }

        let by_id: HashMap<String, AssetPriceMapping> = mappings.into_iter().map(|m| (m.provider_price_id.clone(), m)).collect();
        let ids: Vec<String> = by_id.keys().cloned().collect();
        let mut out = Vec::with_capacity(by_id.len());
        for chunk in ids.chunks(MAX_MARKETS_PER_PAGE) {
            let coin_markets = self.client.get_coin_markets_ids(chunk.to_vec(), MAX_MARKETS_PER_PAGE).await?;
            out.extend(map_coin_markets(coin_markets, &by_id));
        }
        Ok(out)
    }

    async fn get_charts_daily(&self, provider_price_id: &str) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let chart = self.client.get_market_chart(provider_price_id, "daily", "max").await?;
        Ok(map_market_chart_daily(chart))
    }
}
