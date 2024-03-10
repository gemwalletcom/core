use primitives::{Asset, AssetDetails, ChartPeriod, ChartValue};
use redis::{AsyncCommands, RedisResult};
use std::{collections::HashMap, error::Error};
use storage::{
    models::{ChartCoinPrice, FiatRate},
    ClickhouseDatabase, DatabaseClient,
};

use storage::models::Price;

use crate::DEFAULT_FIAT_CURRENCY;

pub struct PriceClient {
    redis_client: redis::Client,
    database: DatabaseClient,
    clickhouse_database: ClickhouseDatabase,
    prefix: String,
}

impl PriceClient {
    pub fn new(redis_url: &str, database_url: &str, clichouse_database_url: &str) -> Self {
        let redis_client = redis::Client::open(redis_url).unwrap();
        let database = DatabaseClient::new(database_url);
        let clickhouse_database = ClickhouseDatabase::new(clichouse_database_url);
        Self {
            redis_client,
            database,
            clickhouse_database,
            prefix: "prices:".to_owned(),
        }
    }

    // db

    pub fn get_coin_id(&mut self, asset_id: &str) -> Result<String, Box<dyn Error>> {
        let prices = self.database.get_prices_for_asset_id(asset_id)?;
        let price = prices.first().ok_or("no price for asset_id")?;
        Ok(price.coin_id.clone())
    }

    pub fn set_prices(&mut self, prices: Vec<Price>) -> Result<usize, Box<dyn Error>> {
        for chunk in prices.chunks(1000).clone() {
            self.database.set_prices(chunk.to_vec())?;
        }
        Ok(prices.len())
    }

    pub fn get_prices(&mut self) -> Result<Vec<Price>, Box<dyn Error>> {
        Ok(self.database.get_prices()?)
    }

    pub async fn set_fiat_rates(&mut self, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error>> {
        Ok(self.database.set_fiat_rates(rates)?)
    }

    pub fn get_fiat_rates(&mut self) -> Result<Vec<FiatRate>, Box<dyn Error>> {
        Ok(self.database.get_fiat_rates()?)
    }

    pub async fn set_charts(
        &mut self,
        charts: Vec<ChartCoinPrice>,
    ) -> Result<usize, Box<dyn Error>> {
        let _ = self.clickhouse_database.add_charts(charts).await?;
        Ok(0)
    }

    pub async fn get_charts_prices(
        &mut self,
        coin_id: &str,
        period: ChartPeriod,
        currency: &str,
    ) -> Result<Vec<ChartValue>, Box<dyn Error>> {
        let base_rate = self.database.get_fiat_rate(DEFAULT_FIAT_CURRENCY)?;
        let rate = self.database.get_fiat_rate(currency)?;
        let rate_multiplier = rate.rate / base_rate.rate;
        let interval = self.period_sql(period.clone());
        let prices = self
            .clickhouse_database
            .get_charts(coin_id, interval, period.minutes())
            .await?
            .into_iter()
            .map(|x| ChartValue {
                timestamp: x.date,
                value: x.price * rate_multiplier,
            })
            .collect();

        Ok(prices)
    }

    fn period_sql(&self, period: ChartPeriod) -> &str {
        match period {
            ChartPeriod::Hour => "1 minute",
            ChartPeriod::Day => "15 minute",
            ChartPeriod::Week => "1 hour",
            ChartPeriod::Month => "6 hour",
            ChartPeriod::Quarter => "1 day",
            ChartPeriod::Year => "3 day",
            ChartPeriod::All => "3 day",
        }
    }

    // cache

    pub fn convert_asset_price_vec_to_map(coins: Vec<Price>) -> HashMap<String, Price> {
        coins
            .into_iter()
            .map(|coin| (coin.asset_id.clone(), coin))
            .collect()
    }

    pub fn asset_key(&mut self, currency: &str, asset: String) -> String {
        format!("{}{}:{}", self.prefix, currency, asset)
    }

    pub async fn set_cache_prices(
        &mut self,
        currency: &str,
        prices: Vec<Price>,
    ) -> RedisResult<usize> {
        let serialized: Vec<(String, String)> = prices
            .iter()
            .map(|x| {
                (
                    self.asset_key(currency, x.asset_id.clone()),
                    serde_json::to_string(x).unwrap(),
                )
            })
            .collect();

        self.redis_client
            .get_multiplexed_async_connection()
            .await?
            .mset(serialized.as_slice())
            .await?;

        Ok(serialized.len())
    }

    pub async fn get_cache_prices(
        &mut self,
        currency: &str,
        assets: Vec<&str>,
    ) -> RedisResult<Vec<Price>> {
        let keys: Vec<String> = assets
            .iter()
            .map(|x| self.asset_key(currency, x.to_string()))
            .collect();
        let result: Vec<Option<String>> = self
            .redis_client
            .get_multiplexed_async_connection()
            .await?
            .mget(keys)
            .await?;

        let values: Vec<String> = result.into_iter().flatten().collect();
        let prices: Vec<Price> = values
            .iter()
            .filter_map(|x| serde_json::from_str(x).unwrap_or(None))
            .collect();

        Ok(prices)
    }

    // asset, asset details
    pub async fn update_asset(
        &mut self,
        asset: Asset,
        asset_details: AssetDetails,
    ) -> Result<(), Box<dyn Error>> {
        let details = storage::models::asset::AssetDetail::from_primitive(
            asset.id.to_string().as_str(),
            asset_details,
        );
        let asset = storage::models::asset::Asset::from_primitive(asset);
        let _ = self.database.add_assets(vec![asset]);
        let _ = self.database.add_assets_details(vec![details]);
        Ok(())
    }
}
