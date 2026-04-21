use cacher::{CacheError, CacheKey, CacherClient};
use primitives::{AssetMarketPrice, AssetPriceInfo, AssetPrices, ChartTimeframe, FiatRate};
use std::error::Error;
use storage::{ChartsRepository, Database, PricesRepository};

#[derive(Clone)]
pub struct PriceClient {
    database: Database,
    cacher_client: CacherClient,
}

impl PriceClient {
    pub fn new(database: Database, cacher_client: CacherClient) -> Self {
        Self { database, cacher_client }
    }

    pub async fn set_fiat_rates(&self, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let count = self
            .database
            .fiat()?
            .set_fiat_rates(rates.clone().into_iter().map(storage::models::FiatRateRow::from_primitive).collect())?;

        self.set_cache_fiat_rates(rates).await?;

        Ok(count)
    }

    pub fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.fiat()?.get_fiat_rates()?.into_iter().map(|r| r.as_primitive()).collect())
    }

    pub fn get_fiat_rate(&self, symbol: &str) -> Result<FiatRate, Box<dyn Error + Send + Sync>> {
        Ok(self.database.fiat()?.get_fiat_rate(symbol)?.as_primitive())
    }

    pub async fn get_asset_price(&self, asset_id: &str, currency: &str) -> Result<AssetMarketPrice, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency)?.rate;
        let price = self.get_cache_price(asset_id).await?;
        let prices = self
            .database
            .prices()?
            .get_prices_for_asset(asset_id)?
            .into_iter()
            .map(|row| row.as_primitive().with_rate(rate))
            .collect();
        Ok(AssetMarketPrice {
            price: Some(price.as_price_primitive_with_rate(rate)),
            market: Some(price.as_market_with_rate(rate)),
            prices: Some(prices),
        })
    }

    pub async fn set_cache_fiat_rates(&self, rates: Vec<FiatRate>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher_client.set_cached(CacheKey::FiatRates, &rates).await
    }

    pub async fn get_cache_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        match self.cacher_client.get_cached_optional::<Vec<FiatRate>>(CacheKey::FiatRates).await? {
            Some(rates) => Ok(rates),
            None => Err(Box::new(CacheError::not_found_resource("FiatRates"))),
        }
    }

    pub async fn set_cache_prices(&self, prices: Vec<AssetPriceInfo>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values: Vec<(String, String)> = prices
            .iter()
            .map(|x| (CacheKey::Price(&x.asset_id.to_string()).key(), serde_json::to_string(&x).unwrap()))
            .collect();

        self.cacher_client.set_values_with_publish(values, ttl_seconds).await
    }

    pub async fn get_cache_prices(&self, asset_ids: Vec<String>) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        let keys: Vec<String> = asset_ids.iter().map(|x| CacheKey::Price(x).key()).collect();
        self.cacher_client.get_values(keys).await
    }

    pub async fn get_cache_price(&self, asset_id: &str) -> Result<AssetPriceInfo, Box<dyn Error + Send + Sync>> {
        match self.cacher_client.get_cached_optional::<AssetPriceInfo>(CacheKey::Price(asset_id)).await? {
            Some(price) => Ok(price),
            None => Err(Box::new(CacheError::not_found("Price", asset_id.to_string()))),
        }
    }

    pub async fn get_asset_prices(&self, currency: &str, asset_ids: Vec<String>) -> Result<AssetPrices, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency)?.rate;
        let prices = self
            .get_cache_prices(asset_ids)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|x| x.as_asset_price_primitive_with_rate(rate))
            .collect();

        Ok(AssetPrices {
            currency: currency.to_string(),
            prices,
        })
    }

    pub async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.charts()?.aggregate_charts(timeframe)?)
    }

    pub async fn cleanup_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.charts()?.cleanup_charts(timeframe)?)
    }

    pub async fn track_observed_assets(&self, asset_ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = CacheKey::ObservedAssets;
        self.cacher_client.sorted_set_incr_with_expire(&key.key(), asset_ids, key.ttl() as i64).await
    }
}
