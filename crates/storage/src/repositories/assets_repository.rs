use std::error::Error;

use crate::{models::Asset, DatabaseClient};
use crate::database::assets::AssetsStore;
use crate::database::assets::{AssetUpdate, AssetFilter};
use primitives::{Asset as PrimitiveAsset, AssetBasic};

pub trait AssetsRepository {
    fn get_assets_all(&mut self) -> Result<Vec<PrimitiveAsset>, Box<dyn Error + Send + Sync>>;
    fn add_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_assets(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn upsert_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>>;
}

impl AssetsRepository for DatabaseClient {
    fn get_assets_all(&mut self) -> Result<Vec<PrimitiveAsset>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_assets_all(self)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }

    fn add_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::add_assets(
            self,
            values.into_iter().map(Asset::from_primitive_default).collect(),
        )?)
    }

    fn update_assets(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::update_assets(self, asset_ids, update)?)
    }

    fn upsert_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::upsert_assets(
            self,
            values.into_iter().map(Asset::from_primitive_default).collect(),
        )?)
    }

    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_assets_by_filter(self, filters)?
            .into_iter()
            .map(|x| x.as_basic_primitive())
            .collect())
    }
}