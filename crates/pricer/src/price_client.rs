use chrono::NaiveDateTime;
use primitives::{AssetMarketPrice, AssetPriceInfo, AssetPrices, FiatRate};
use std::error::Error;
use storage::{
    models::{Chart, Price, PriceAsset},
    DatabaseClient,
};

use cacher::CacherClient;

pub struct PriceClient {
    cacher_client: CacherClient,
    database: DatabaseClient,
}

const PRICES_INSERT_BATCH_LIMIT: usize = 1000;
const PRICES_ASSETS_INSERT_BATCH_LIMIT: usize = 1000;
const FIAT_RATES_KEY: &str = "fiat_rates";

impl PriceClient {
    pub fn new(cacher_client: CacherClient, database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { cacher_client, database }
    }

    // db

    pub fn get_coin_id(&mut self, asset_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let prices = self.database.prices().get_prices_assets_for_asset_id(asset_id)?;
        let price = prices.first().ok_or("no price for asset_id")?;
        Ok(price.price_id.clone())
    }

    pub fn set_prices(&mut self, prices: Vec<Price>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        for chunk in prices.chunks(PRICES_INSERT_BATCH_LIMIT).clone() {
            self.database.prices().set_prices(chunk.to_vec())?;
        }
        Ok(prices.len())
    }

    pub fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        // filter non existing prices and assets
        let assets_ids = self
            .database
            .assets()
            .get_assets(values.iter().map(|x| x.asset_id.clone()).collect())?
            .iter()
            .map(|x| x.id.to_string())
            .collect::<Vec<_>>();

        let prices_ids = self.database.prices().get_prices()?.iter().map(|x| x.id.clone()).collect::<Vec<_>>();

        let values = values
            .into_iter()
            .filter(|x| assets_ids.contains(&x.asset_id) && prices_ids.contains(&x.price_id))
            .collect::<Vec<_>>();

        for chunk in values.chunks(PRICES_ASSETS_INSERT_BATCH_LIMIT).clone() {
            self.database.prices().set_prices_assets(chunk.to_vec())?;
        }
        Ok(values.len())
    }

    pub fn get_prices(&mut self) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>> {
        self.database.prices().get_prices()
    }

    pub fn get_prices_ids(&mut self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.prices().get_prices()?.into_iter().map(|x| x.id).collect())
    }

    pub fn get_prices_assets(&mut self) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>> {
        self.database.prices().get_prices_assets()
    }

    pub async fn set_fiat_rates(&mut self, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.database
            .fiat()
            .set_fiat_rates(rates.into_iter().map(storage::models::FiatRate::from_primitive).collect())
    }

    pub fn get_fiat_rates(&mut self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        self.database.fiat().get_fiat_rates()
    }

    pub fn get_fiat_rate(&mut self, symbol: &str) -> Result<FiatRate, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .fiat()
            .get_fiat_rates()?
            .iter()
            .find(|x| x.symbol == symbol)
            .ok_or(format!("No fiat rate found for symbol: {symbol}"))?
            .clone())
    }

    // cache

    pub async fn get_asset_price(&mut self, asset_id: &str, currency: &str) -> Result<AssetMarketPrice, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency)?.rate;
        let prices = self.get_cache_prices(vec![asset_id.to_string()]).await?;
        let price = prices.first().cloned().ok_or(format!("No price available for asset_id: {asset_id}"))?;

        Ok(AssetMarketPrice {
            price: Some(price.as_price_primitive_with_rate(rate)),
            market: Some(price.as_market_with_rate(rate)),
        })
    }

    pub async fn set_cache_fiat_rates(&mut self, rates: Vec<FiatRate>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher_client.set_value(FIAT_RATES_KEY, &rates).await
    }

    pub async fn get_cache_fiat_rates(&mut self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        self.cacher_client.get_value(FIAT_RATES_KEY).await
    }

    pub async fn set_cache_prices(&mut self, prices: Vec<AssetPriceInfo>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values: Vec<(String, String)> = prices
            .iter()
            .map(|x| (x.asset_id.to_string().clone(), serde_json::to_string(&x).unwrap()))
            .collect();

        self.cacher_client.set_values_with_publish(values, ttl_seconds).await
    }

    pub async fn get_cache_prices(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        let keys: Vec<String> = asset_ids.iter().map(|x| x.to_string()).collect();
        self.cacher_client.get_values(keys).await
    }

    pub async fn get_asset_prices(&mut self, currency: &str, asset_ids: Vec<String>) -> Result<AssetPrices, Box<dyn Error + Send + Sync>> {
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

    // charts
    pub async fn add_charts(&mut self, charts: Vec<Chart>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.database.charts().add_charts(charts)
    }

    pub fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.database.prices().delete_prices_updated_at_before(time)
    }

    pub async fn aggregate_hourly_charts(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.database.charts().aggregate_hourly_charts()
    }

    pub async fn aggregate_daily_charts(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.database.charts().aggregate_daily_charts()
    }

    pub async fn cleanup_charts_data(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.database.charts().cleanup_charts_data()
    }
}
