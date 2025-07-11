use crate::schema::assets::dsl::*;

use crate::{models::Asset, DatabaseClient};
use diesel::{prelude::*, upsert::excluded};

#[derive(Debug, Clone)]
pub enum AssetUpdate {
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
    Rank(i32),
}

#[derive(Debug, Clone)]
pub enum AssetFilter {
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
}

pub(crate) trait AssetsStore {
    fn get_assets_all(&mut self) -> Result<Vec<Asset>, diesel::result::Error>;
    fn add_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error>;
    fn update_assets(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, diesel::result::Error>;
    fn upsert_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<Asset>, diesel::result::Error>;
}

impl AssetsStore for DatabaseClient {
    fn get_assets_all(&mut self) -> Result<Vec<Asset>, diesel::result::Error> {
        assets.filter(is_enabled.eq(true)).select(Asset::as_select()).load(&mut self.connection)
    }

    fn add_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        diesel::insert_into(assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn update_assets(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, diesel::result::Error> {
        if asset_ids.is_empty() {
            return Ok(0);
        }
        
        let target = assets.filter(id.eq_any(&asset_ids));
        
        match update {
            AssetUpdate::IsSwappable(value) => {
                diesel::update(target).set(is_swappable.eq(value)).execute(&mut self.connection)
            }
            AssetUpdate::IsBuyable(value) => {
                diesel::update(target).set(is_buyable.eq(value)).execute(&mut self.connection)
            }
            AssetUpdate::IsSellable(value) => {
                diesel::update(target).set(is_sellable.eq(value)).execute(&mut self.connection)
            }
            AssetUpdate::Rank(value) => {
                diesel::update(target).set(rank.eq(value)).execute(&mut self.connection)
            }
        }
    }

    fn upsert_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(assets)
            .values(values)
            .on_conflict(id)
            .do_update()
            .set((rank.eq(excluded(rank)),))
            .execute(&mut self.connection)
    }

    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<Asset>, diesel::result::Error> {
        let mut query = assets.filter(is_enabled.eq(true)).into_boxed();
        
        for filter in filters {
            match filter {
                AssetFilter::IsBuyable(value) => {
                    query = query.filter(is_buyable.eq(value));
                }
                AssetFilter::IsSellable(value) => {
                    query = query.filter(is_sellable.eq(value));
                }
                AssetFilter::IsSwappable(value) => {
                    query = query.filter(is_swappable.eq(value));
                }
            }
        }
        
        query.select(Asset::as_select()).load(&mut self.connection)
    }
}
