use crate::schema::assets::dsl::*;

use crate::{models::Asset, DatabaseClient};
use diesel::{prelude::*, upsert::excluded};

impl DatabaseClient {
    pub fn get_assets_list(&mut self) -> Result<Vec<Asset>, diesel::result::Error> {
        assets.filter(is_enabled.eq(true)).select(Asset::as_select()).load(&mut self.connection)
    }

    pub fn add_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        diesel::insert_into(assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn update_assets_rank(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(assets)
            .values(values)
            .on_conflict(id)
            .do_update()
            .set((rank.eq(excluded(rank)),))
            .execute(&mut self.connection)
    }

    pub fn get_assets_is_buyable(&mut self) -> Result<Vec<Asset>, diesel::result::Error> {
        assets.filter(is_buyable.eq(true)).select(Asset::as_select()).load(&mut self.connection)
    }

    pub fn get_assets_is_sellable(&mut self) -> Result<Vec<Asset>, diesel::result::Error> {
        assets.filter(is_sellable.eq(true)).select(Asset::as_select()).load(&mut self.connection)
    }

    pub fn set_assets_is_swap_enabled(&mut self, asset_ids: Vec<String>, value: bool) -> Result<usize, diesel::result::Error> {
        diesel::update(assets)
            .filter(id.eq_any(&asset_ids))
            .set(is_swappable.eq(value))
            .execute(&mut self.connection)
    }

    pub fn set_assets_is_buyable(&mut self, asset_ids: Vec<String>, value: bool) -> Result<usize, diesel::result::Error> {
        diesel::update(assets)
            .filter(id.eq_any(&asset_ids))
            .set(is_buyable.eq(value))
            .execute(&mut self.connection)
    }

    pub fn set_assets_is_sellable(&mut self, asset_ids: Vec<String>, value: bool) -> Result<usize, diesel::result::Error> {
        diesel::update(assets)
            .filter(id.eq_any(&asset_ids))
            .set(is_sellable.eq(value))
            .execute(&mut self.connection)
    }
}
