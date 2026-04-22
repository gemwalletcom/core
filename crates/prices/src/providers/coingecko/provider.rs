use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use coingecko::{client::CoinGeckoClient, get_coingecko_market_id_for_chain};
use gem_client::{Client, ReqwestClient};
use primitives::{Chain, ChartValue, DurationExt};

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset, PriceProviderAssetMetadata};

use super::mapper::{map_coin_info_metadata, map_coin_markets, map_coins_to_assets, map_coins_to_mappings, map_market_chart};

const MAX_MARKETS_PER_PAGE: usize = 250;
const MAX_RANKED_PAGES: usize = 20;

pub struct CoinGeckoPricesProvider<C: Client = ReqwestClient> {
    client: CoinGeckoClient<C>,
}

impl CoinGeckoPricesProvider<ReqwestClient> {
    pub fn new(api_key: &str) -> Self {
        Self::from_client(CoinGeckoClient::new(api_key))
    }
}

impl<C: Client> CoinGeckoPricesProvider<C> {
    pub fn from_client(client: CoinGeckoClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> PriceAssetsProvider for CoinGeckoPricesProvider<C> {
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
        Ok(map_market_chart(chart))
    }

    async fn get_charts_hourly(&self, provider_price_id: &str, duration: Duration) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let days = duration.as_days_ceil().max(1).to_string();
        let chart = self.client.get_market_chart(provider_price_id, "hourly", &days).await?;
        Ok(map_market_chart(chart))
    }

    async fn get_charts_raw(&self, provider_price_id: &str, duration: Duration) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let days = duration.as_days_ceil().max(1).to_string();
        let chart = self.client.get_market_chart_auto(provider_price_id, &days).await?;
        Ok(map_market_chart(chart))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_get_charts_hourly() {
        let paths = Arc::new(Mutex::new(Vec::new()));
        let captured_paths = paths.clone();
        let client = MockClient::new().with_get(move |path| {
            captured_paths.lock().unwrap().push(path.to_string());
            Ok(br#"{"prices":[[1713744000000,123.45]]}"#.to_vec())
        });
        let provider = CoinGeckoPricesProvider::from_client(CoinGeckoClient::new_with_client(client));

        let values = provider.get_charts_hourly("bitcoin", Duration::from_secs(primitives::DAY.as_secs() * 7)).await.unwrap();

        assert_eq!(
            values,
            vec![ChartValue {
                timestamp: 1_713_744_000,
                value: 123.45,
            }]
        );
        assert_eq!(
            paths.lock().unwrap().clone(),
            vec!["/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days=7&interval=hourly&precision=full".to_string()]
        );
    }

    #[tokio::test]
    async fn test_get_charts_raw() {
        let paths = Arc::new(Mutex::new(Vec::new()));
        let captured_paths = paths.clone();
        let client = MockClient::new().with_get(move |path| {
            captured_paths.lock().unwrap().push(path.to_string());
            Ok(br#"{"prices":[[1713744000000,123.45]]}"#.to_vec())
        });
        let provider = CoinGeckoPricesProvider::from_client(CoinGeckoClient::new_with_client(client));

        let values = provider.get_charts_raw("bitcoin", primitives::DAY).await.unwrap();

        assert_eq!(
            values,
            vec![ChartValue {
                timestamp: 1_713_744_000,
                value: 123.45,
            }]
        );
        assert_eq!(
            paths.lock().unwrap().clone(),
            vec!["/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days=1&precision=full".to_string()]
        );
    }
}
