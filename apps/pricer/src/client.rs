use chrono::NaiveDateTime;
use primitives::{Asset, AssetDetails, AssetScore};
use redis::{AsyncCommands, RedisResult};
use std::error::Error;
use storage::{
    models::{
        price::{PriceAsset, PriceCache},
        FiatRate,
    },
    DatabaseClient,
};

use cacher::CacherClient;
use storage::models::Price;

pub struct PriceClient {
    redis_client: redis::Client,
    cache_client: CacherClient,
    database: DatabaseClient,
    prefix: String,
}

const PRICES_INSERT_BATCH_LIMIT: usize = 1000;

impl PriceClient {
    pub fn new(redis_url: &str, database_url: &str) -> Self {
        let redis_client = redis::Client::open(redis_url).unwrap();
        let database = DatabaseClient::new(database_url);
        Self {
            redis_client,
            cache_client: CacherClient::new(redis_url),
            database,
            prefix: "prices:".to_owned(),
        }
    }

    // db

    pub fn get_coin_id(&mut self, asset_id: &str) -> Result<String, Box<dyn Error>> {
        let prices = self.database.get_prices_id_for_asset_id(asset_id)?;
        let price = prices.first().ok_or("no price for asset_id")?;
        Ok(price.price_id.clone())
    }

    pub fn set_prices(&mut self, prices: Vec<Price>) -> Result<usize, Box<dyn Error>> {
        for chunk in prices.chunks(PRICES_INSERT_BATCH_LIMIT).clone() {
            self.database.set_prices(chunk.to_vec())?;
        }
        Ok(prices.len())
    }

    pub fn set_prices_simple(&mut self, prices: Vec<Price>) -> Result<usize, Box<dyn Error>> {
        for chunk in prices.chunks(PRICES_INSERT_BATCH_LIMIT).clone() {
            self.database.set_prices_simple(chunk.to_vec())?;
        }
        Ok(prices.len())
    }

    pub fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, Box<dyn Error>> {
        // filter non existing prices and assets
        let existing_assets_ids = self
            .database
            .get_assets(values.iter().map(|x| x.asset_id.clone()).collect())?
            .iter()
            .map(|x| x.id.clone())
            .collect::<Vec<_>>();

        let existing_prices_ids = self
            .database
            .get_prices()?
            .iter()
            .map(|x| x.id.clone())
            .collect::<Vec<_>>();

        let values = values
            .into_iter()
            .filter(|x| {
                existing_assets_ids.contains(&x.asset_id)
                    && existing_prices_ids.contains(&x.price_id)
            })
            .collect::<Vec<_>>();

        Ok(self.database.set_prices_assets(values)?)
    }

    pub fn get_prices(&mut self) -> Result<Vec<Price>, Box<dyn Error>> {
        Ok(self.database.get_prices()?)
    }

    pub fn get_prices_ids(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self
            .database
            .get_prices()?
            .into_iter()
            .map(|x| x.id)
            .collect())
    }

    pub fn get_prices_assets(&mut self) -> Result<Vec<PriceAsset>, Box<dyn Error>> {
        Ok(self.database.get_prices_assets()?)
    }

    pub async fn set_fiat_rates(&mut self, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error>> {
        Ok(self.database.set_fiat_rates(rates)?)
    }

    pub fn get_fiat_rates(&mut self) -> Result<Vec<FiatRate>, Box<dyn Error>> {
        Ok(self.database.get_fiat_rates()?)
    }

    // cache

    pub fn asset_key(&mut self, currency: &str, asset_id: String) -> String {
        format!("{}{}:{}", self.prefix, currency, asset_id)
    }

    pub async fn set_cache_prices(
        &mut self,
        currency: &str,
        prices: Vec<PriceCache>,
    ) -> Result<usize, Box<dyn Error>> {
        let values: Vec<(String, String)> = prices
            .iter()
            .map(|x| {
                (
                    self.asset_key(currency, x.asset_id.clone()),
                    serde_json::to_string(&x).unwrap(),
                )
            })
            .collect();

        self.cache_client.set_values(values).await
    }

    pub async fn get_cache_prices(
        &mut self,
        currency: &str,
        assets: Vec<&str>,
    ) -> RedisResult<Vec<PriceCache>> {
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

        let prices: Vec<PriceCache> = result
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .iter()
            .filter_map(|x| serde_json::from_str(x).unwrap_or(None))
            .collect();

        Ok(prices)
    }

    // asset, asset details
    pub async fn update_asset(
        &mut self,
        asset: Asset,
        asset_score: AssetScore,
        asset_details: AssetDetails,
    ) -> Result<(), Box<dyn Error>> {
        let details = storage::models::asset::AssetDetail::from_primitive(
            asset.id.to_string().as_str(),
            asset_details,
        );
        let asset = storage::models::asset::Asset::from_primitive(asset);
        let asset_id = asset.id.as_str();
        let _ = self.database.add_assets(vec![asset.clone()]);
        let _ = self.database.add_assets_details(vec![details]);
        let _ = self.database.update_asset_rank(asset_id, asset_score.rank);
        Ok(())
    }

    pub fn delete_prices_updated_at_before(
        &mut self,
        time: NaiveDateTime,
    ) -> Result<usize, Box<dyn Error>> {
        Ok(self.database.delete_prices_updated_at_before(time)?)
    }
}
