use crate::schema::assets::dsl::*;

use crate::{models::Asset, DatabaseClient};
use diesel::{prelude::*, upsert::excluded};

#[derive(Debug, Clone)]
pub enum AssetUpdate {
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
    IsEnabled(bool),
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
    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, diesel::result::Error>;
    fn update_assets_bulk(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, diesel::result::Error>;
    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, diesel::result::Error>;
    fn upsert_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<Asset>, diesel::result::Error>;
    fn get_asset(&mut self, asset_id: &str) -> Result<Asset, diesel::result::Error>;
    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<Asset>, diesel::result::Error>;
    fn get_swap_assets(&mut self) -> Result<Vec<String>, diesel::result::Error>;
    fn get_swap_assets_version(&mut self) -> Result<i32, diesel::result::Error>;
    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, diesel::result::Error>;
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

    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, diesel::result::Error> {
        if asset_ids.is_empty() || updates.is_empty() || asset_ids.len() != updates.len() {
            return Ok(0);
        }

        self.connection.transaction(|conn| {
            let mut total_updated = 0;

            for (asset_id, update) in asset_ids.into_iter().zip(updates.into_iter()) {
                let target = assets.filter(id.eq(&asset_id));
                let updated = match update {
                    AssetUpdate::IsSwappable(value) => diesel::update(target).set(is_swappable.eq(value)).execute(conn)?,
                    AssetUpdate::IsBuyable(value) => diesel::update(target).set(is_buyable.eq(value)).execute(conn)?,
                    AssetUpdate::IsSellable(value) => diesel::update(target).set(is_sellable.eq(value)).execute(conn)?,
                    AssetUpdate::IsEnabled(value) => diesel::update(target).set(is_enabled.eq(value)).execute(conn)?,
                    AssetUpdate::Rank(value) => diesel::update(target).set(rank.eq(value)).execute(conn)?,
                };
                total_updated += updated;
            }

            Ok(total_updated)
        })
    }

    fn update_assets_bulk(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, diesel::result::Error> {
        let updates = vec![update; asset_ids.len()];
        self.update_assets(asset_ids, updates)
    }

    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, diesel::result::Error> {
        let target = assets.filter(id.eq(&asset_id));

        match update {
            AssetUpdate::IsSwappable(value) => diesel::update(target).set(is_swappable.eq(value)).execute(&mut self.connection),
            AssetUpdate::IsBuyable(value) => diesel::update(target).set(is_buyable.eq(value)).execute(&mut self.connection),
            AssetUpdate::IsSellable(value) => diesel::update(target).set(is_sellable.eq(value)).execute(&mut self.connection),
            AssetUpdate::IsEnabled(value) => diesel::update(target).set(is_enabled.eq(value)).execute(&mut self.connection),
            AssetUpdate::Rank(value) => diesel::update(target).set(rank.eq(value)).execute(&mut self.connection),
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

    fn get_asset(&mut self, asset_id: &str) -> Result<Asset, diesel::result::Error> {
        assets.filter(id.eq(asset_id)).select(Asset::as_select()).first(&mut self.connection)
    }

    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<Asset>, diesel::result::Error> {
        assets.filter(id.eq_any(asset_ids)).select(Asset::as_select()).load(&mut self.connection)
    }

    fn get_swap_assets(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        assets
            .filter(rank.gt(21))
            .filter(is_swappable.eq(true))
            .select(id)
            .order(rank.desc())
            .load(&mut self.connection)
    }

    fn get_swap_assets_version(&mut self) -> Result<i32, diesel::result::Error> {
        Ok((std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() / 3600) as i32)
    }

    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, diesel::result::Error> {
        let chain_values = values.iter().map(|chain_id| crate::models::Chain { id: chain_id.clone() }).collect::<Vec<_>>();

        use crate::schema::chains::dsl::*;
        diesel::insert_into(chains)
            .values(chain_values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}
