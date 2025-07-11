use std::error::Error;
use chrono::NaiveDateTime;

use crate::DatabaseClient;
use crate::database::prices::PricesStore;
use crate::models::{Price, PriceAsset, price::PriceAssetData};

pub trait PricesRepository {
    fn set_prices(&mut self, values: Vec<Price>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_prices(&mut self) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>>;
    fn get_prices_assets(&mut self) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>>;
    fn get_price(&mut self, asset_id: &str) -> Result<Price, Box<dyn Error + Send + Sync>>;
    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>>;
    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_prices_assets_list(&mut self) -> Result<Vec<PriceAssetData>, Box<dyn Error + Send + Sync>>;
}

impl PricesRepository for DatabaseClient {
    fn set_prices(&mut self, values: Vec<Price>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::set_prices(self, values)?)
    }

    fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::set_prices_assets(self, values)?)
    }

    fn get_prices(&mut self) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::get_prices(self)?)
    }

    fn get_prices_assets(&mut self) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::get_prices_assets(self)?)
    }

    fn get_price(&mut self, asset_id: &str) -> Result<Price, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::get_price(self, asset_id)?)
    }

    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::get_prices_assets_for_asset_id(self, id)?)
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAsset>, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::get_prices_assets_for_price_ids(self, ids)?)
    }

    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::delete_prices_updated_at_before(self, time)?)
    }

    fn get_prices_assets_list(&mut self) -> Result<Vec<PriceAssetData>, Box<dyn Error + Send + Sync>> {
        Ok(PricesStore::get_prices_assets_list(self)?)
    }
}