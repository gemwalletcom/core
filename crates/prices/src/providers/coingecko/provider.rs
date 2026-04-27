use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use coingecko::{client::CoinGeckoClient, get_coingecko_market_id_for_chain, get_coingecko_platform_id_for_chain};
use gem_client::ReqwestClient;
use primitives::{AssetId, Chain, ChartValue, DurationExt};

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset, PriceProviderAssetMetadata, PriceProviderConfig};

use super::mapper::{map_coin_info_metadata, map_coin_mappings, map_coin_markets, map_coins_to_assets, map_coins_to_mappings, map_market_chart};

const MAX_MARKETS_PER_PAGE: usize = 250;
const MAX_RANKED_PAGES: usize = 20;

pub struct CoinGeckoPricesProvider {
    client: CoinGeckoClient<ReqwestClient>,
}

impl CoinGeckoPricesProvider {
    pub fn new(api_key: &str, _config: PriceProviderConfig) -> Self {
        Self {
            client: CoinGeckoClient::new(api_key),
        }
    }
}

#[async_trait]
impl PriceAssetsProvider for CoinGeckoPricesProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Coingecko
    }

    async fn get_assets(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let mut markets_by_id: HashMap<String, _> = self
            .client
            .get_all_coin_markets(None, MAX_MARKETS_PER_PAGE, MAX_RANKED_PAGES)
            .await?
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        let native_ids: Vec<String> = Chain::all()
            .into_iter()
            .map(get_coingecko_market_id_for_chain)
            .map(str::to_string)
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|id| !markets_by_id.contains_key(id))
            .collect();
        if !native_ids.is_empty() {
            let native_markets = self.client.get_coin_markets_ids(native_ids, MAX_MARKETS_PER_PAGE).await?;
            markets_by_id.extend(native_markets.into_iter().map(|market| (market.id.clone(), market)));
        }

        let coins = self.client.get_coin_list().await?.into_iter().filter(|c| markets_by_id.contains_key(&c.id)).collect();
        Ok(map_coins_to_assets(coins, markets_by_id))
    }

    async fn get_assets_new(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let ids: HashSet<String> = self
            .client
            .get_search_trending()
            .await?
            .get_coins_ids()
            .into_iter()
            .chain(self.client.get_coin_list_new().await?.ids())
            .collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let coins = self.client.get_coin_list().await?.into_iter().filter(|c| ids.contains(&c.id)).collect();
        Ok(map_coins_to_mappings(coins).into_iter().map(|m| PriceProviderAsset::new(m, None)).collect())
    }

    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        let (Some(platform_id), Some(token_id)) = (get_coingecko_platform_id_for_chain(asset_id.chain), asset_id.token_id.as_deref()) else {
            return Ok(vec![]);
        };
        let coin_info = self.client.get_coin_by_contract(platform_id, token_id).await?;
        Ok(vec![AssetPriceMapping::new(asset_id.clone(), coin_info.id)])
    }

    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        let coin_info = self.client.get_coin(provider_price_id).await?;
        Ok(map_coin_mappings(&coin_info.id, &coin_info.platforms))
    }

    async fn get_assets_metadata(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<PriceProviderAssetMetadata>, Box<dyn Error + Send + Sync>> {
        let grouped = mappings.into_iter().fold(HashMap::new(), |mut grouped, mapping| {
            grouped.entry(mapping.provider_price_id.clone()).or_insert_with(Vec::new).push(mapping);
            grouped
        });
        let mut metadata = Vec::new();
        for (provider_price_id, mappings) in grouped {
            let coin_info = self.client.get_coin(&provider_price_id).await?;
            metadata.extend(map_coin_info_metadata(mappings, coin_info));
        }
        Ok(metadata)
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }

        let by_id = mappings.into_iter().fold(HashMap::<String, Vec<AssetPriceMapping>>::new(), |mut by_id, mapping| {
            by_id.entry(mapping.provider_price_id.clone()).or_default().push(mapping);
            by_id
        });
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
        Ok(map_market_chart(chart))
    }

    async fn get_charts_hourly(&self, provider_price_id: &str, duration: Duration) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let days = duration.as_days_ceil().max(1).to_string();
        let chart = self.client.get_market_chart(provider_price_id, "hourly", &days).await?;
        Ok(map_market_chart(chart))
    }
}
