use crate::{DatabaseError, DieselResultExt};
use chrono::NaiveDateTime;
use primitives::Price;

use crate::DatabaseClient;
use crate::database::prices::{AssetsWithPricesFilter, PricesStore};
use crate::models::{PriceAssetRow, PriceRow, price::NewPriceRow, price::PriceAssetDataRow};

pub trait PricesRepository {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, DatabaseError>;
    fn set_prices(&mut self, values: Vec<PriceRow>) -> Result<usize, DatabaseError>;
    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, DatabaseError>;
    fn get_prices(&mut self) -> Result<Vec<PriceRow>, DatabaseError>;
    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn get_price(&mut self, asset_id: &str) -> Result<Price, DatabaseError>;
    fn get_coin_id(&mut self, asset_id: &str) -> Result<String, DatabaseError>;
    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, DatabaseError>;
    fn get_assets_with_prices_by_filter(&mut self, filters: Vec<AssetsWithPricesFilter>) -> Result<Vec<PriceAssetDataRow>, DatabaseError>;
}

impl PricesRepository for DatabaseClient {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::add_prices(self, values)?)
    }
    fn set_prices(&mut self, values: Vec<PriceRow>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::set_prices(self, values)?)
    }

    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::set_prices_assets(self, values)?)
    }

    fn get_prices(&mut self) -> Result<Vec<PriceRow>, DatabaseError> {
        Ok(PricesStore::get_prices(self)?)
    }

    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, DatabaseError> {
        Ok(PricesStore::get_prices_assets(self)?)
    }

    fn get_price(&mut self, asset_id: &str) -> Result<Price, DatabaseError> {
        Ok(PricesStore::get_price(self, asset_id).or_not_found(asset_id.to_string())?.as_primitive())
    }

    fn get_coin_id(&mut self, asset_id: &str) -> Result<String, DatabaseError> {
        Ok(PricesStore::get_price(self, asset_id).or_not_found(asset_id.to_string())?.id)
    }

    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAssetRow>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_for_asset_id(self, id)?)
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_for_price_ids(self, ids)?)
    }

    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(PricesStore::delete_prices_updated_at_before(self, time)?)
    }

    fn get_assets_with_prices_by_filter(&mut self, filters: Vec<AssetsWithPricesFilter>) -> Result<Vec<PriceAssetDataRow>, DatabaseError> {
        Ok(PricesStore::get_assets_with_prices_by_filter(self, filters)?)
    }
}
