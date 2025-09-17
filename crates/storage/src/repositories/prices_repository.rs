use crate::DatabaseError;
use chrono::NaiveDateTime;

use crate::database::prices::PricesStore;
use crate::models::{price::PriceAssetData, Price, PriceAsset};
use crate::DatabaseClient;

pub trait PricesRepository {
    fn set_prices(&mut self, values: Vec<Price>) -> Result<usize, DatabaseError>;
    fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, DatabaseError>;
    fn get_prices(&mut self) -> Result<Vec<Price>, DatabaseError>;
    fn get_prices_assets(&mut self) -> Result<Vec<PriceAsset>, DatabaseError>;
    fn get_price(&mut self, asset_id: &str) -> Result<Option<Price>, DatabaseError>;
    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAsset>, DatabaseError>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError>;
    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, DatabaseError>;
    fn get_prices_assets_list(&mut self) -> Result<Vec<PriceAssetData>, DatabaseError>;
}

impl PricesRepository for DatabaseClient {
    fn set_prices(&mut self, values: Vec<Price>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::set_prices(self, values)?)
    }

    fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::set_prices_assets(self, values)?)
    }

    fn get_prices(&mut self) -> Result<Vec<Price>, DatabaseError> {
        Ok(PricesStore::get_prices(self)?)
    }

    fn get_prices_assets(&mut self) -> Result<Vec<PriceAsset>, DatabaseError> {
        Ok(PricesStore::get_prices_assets(self)?)
    }

    fn get_price(&mut self, asset_id: &str) -> Result<Option<Price>, DatabaseError> {
        Ok(PricesStore::get_price(self, asset_id)?)
    }

    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAsset>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_for_asset_id(self, id)?)
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_for_price_ids(self, ids)?)
    }

    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(PricesStore::delete_prices_updated_at_before(self, time)?)
    }

    fn get_prices_assets_list(&mut self) -> Result<Vec<PriceAssetData>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_list(self)?)
    }
}
