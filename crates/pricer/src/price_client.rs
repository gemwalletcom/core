use chrono::NaiveDateTime;
use primitives::{AssetMarketPrice, AssetPriceInfo, AssetPrices, FiatRate};
use std::error::Error;
use storage::{
    Database,
    models::{Chart, Price, PriceAsset},
};

use cacher::CacherClient;

#[derive(Clone)]
pub struct PriceClient {
    database: Database,
    cacher_client: CacherClient,
}

const PRICES_INSERT_BATCH_LIMIT: usize = 1000;
const FIAT_RATES_KEY: &str = "fiat_rates";

impl PriceClient {
    pub fn new(database: Database, cacher_client: CacherClient) -> Self {
        Self { database, cacher_client }
    }

    pub fn get_coin_id(&self, asset_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let prices = self.database.client()?.prices().get_prices_assets_for_asset_id(asset_id)?;
        let price = prices.first().ok_or("no price for asset_id")?;
        Ok(price.price_id.clone())
    }

    pub fn set_prices(&self, prices: Vec<Price>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        for chunk in prices.chunks(PRICES_INSERT_BATCH_LIMIT).clone() {
            self.database.client()?.prices().set_prices(chunk.to_vec())?;
        }
        Ok(prices.len())
    }

    pub fn get_prices(&self) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.prices().get_prices()?)
    }

    pub fn get_prices_ids(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.prices().get_prices()?.into_iter().map(|x| x.id).collect())
    }

    pub fn get_prices_assets(&self) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.prices().get_prices_assets()?)
    }

    pub async fn set_fiat_rates(&self, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let count = self
            .database
            .client()?
            .fiat()
            .set_fiat_rates(rates.clone().into_iter().map(storage::models::FiatRate::from_primitive).collect())?;

        self.set_cache_fiat_rates(rates).await?;

        Ok(count)
    }

    pub fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.fiat().get_fiat_rates()?)
    }

    pub fn get_fiat_rate(&self, symbol: &str) -> Result<FiatRate, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.fiat().get_fiat_rate(symbol)?)
    }

    pub async fn get_asset_price(&self, asset_id: &str, currency: &str) -> Result<AssetMarketPrice, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency)?.rate;
        let price = self.get_cache_price(asset_id).await?;

        Ok(AssetMarketPrice {
            price: Some(price.as_price_primitive_with_rate(rate)),
            market: Some(price.as_market_with_rate(rate)),
        })
    }

    pub async fn set_cache_fiat_rates(&self, rates: Vec<FiatRate>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher_client.set_value(FIAT_RATES_KEY, &rates).await
    }

    pub async fn get_cache_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        self.cacher_client.get_value(FIAT_RATES_KEY).await
    }

    pub async fn set_cache_prices(&self, prices: Vec<AssetPriceInfo>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values: Vec<(String, String)> = prices
            .iter()
            .map(|x| (x.asset_id.to_string().clone(), serde_json::to_string(&x).unwrap()))
            .collect();

        self.cacher_client.set_values_with_publish(values, ttl_seconds).await
    }

    pub async fn get_cache_prices(&self, asset_ids: Vec<String>) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        let keys: Vec<String> = asset_ids.iter().map(|x| x.to_string()).collect();
        self.cacher_client.get_values(keys).await
    }

    pub async fn get_cache_price(&self, asset_id: &str) -> Result<AssetPriceInfo, Box<dyn Error + Send + Sync>> {
        self.cacher_client.get_value(asset_id).await
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

    pub async fn add_charts(&self, charts: Vec<Chart>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.charts().add_charts(charts)?)
    }

    pub fn delete_prices_updated_at_before(&self, time: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.prices().delete_prices_updated_at_before(time)?)
    }

    pub async fn aggregate_hourly_charts(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.charts().aggregate_hourly_charts()?)
    }

    pub async fn aggregate_daily_charts(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.charts().aggregate_daily_charts()?)
    }

    pub async fn cleanup_charts_data(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.charts().cleanup_charts_data()?)
    }
}
