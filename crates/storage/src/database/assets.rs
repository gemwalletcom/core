use crate::schema::assets::dsl::*;

use crate::{DatabaseClient, models::Asset, models::Price};
use diesel::{prelude::*, upsert::excluded};

#[derive(Debug, Clone)]
pub enum AssetUpdate {
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
    IsEnabled(bool),
    Rank(i32),
    StakingApr(Option<f64>),
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
    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, diesel::result::Error>;
    fn upsert_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<Asset>, diesel::result::Error>;
    fn get_asset(&mut self, asset_id: &str) -> Result<Asset, diesel::result::Error>;
    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<Asset>, diesel::result::Error>;
    fn get_assets_with_prices(&mut self, asset_ids: Vec<String>) -> Result<Vec<(Asset, Option<Price>)>, diesel::result::Error>;
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
        if asset_ids.is_empty() || updates.is_empty() {
            return Ok(0);
        }

        self.connection.transaction(|conn| {
            let mut total_updated = 0;

            for asset_id in asset_ids.into_iter() {
                for update in &updates {
                    let target = assets.find(&asset_id);
                    let updated = match update {
                        AssetUpdate::IsSwappable(value) => diesel::update(target).set(is_swappable.eq(*value)).execute(conn)?,
                        AssetUpdate::IsBuyable(value) => diesel::update(target).set(is_buyable.eq(*value)).execute(conn)?,
                        AssetUpdate::IsSellable(value) => diesel::update(target).set(is_sellable.eq(*value)).execute(conn)?,
                        AssetUpdate::IsEnabled(value) => diesel::update(target).set(is_enabled.eq(*value)).execute(conn)?,
                        AssetUpdate::Rank(value) => diesel::update(target).set(rank.eq(*value)).execute(conn)?,
                        AssetUpdate::StakingApr(value) => diesel::update(target).set(staking_apr.eq(*value)).execute(conn)?,
                    };
                    total_updated += updated;
                }
            }

            Ok(total_updated)
        })
    }

    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, diesel::result::Error> {
        let target = assets.find(&asset_id);

        match update {
            AssetUpdate::IsSwappable(value) => diesel::update(target).set(is_swappable.eq(value)).execute(&mut self.connection),
            AssetUpdate::IsBuyable(value) => diesel::update(target).set(is_buyable.eq(value)).execute(&mut self.connection),
            AssetUpdate::IsSellable(value) => diesel::update(target).set(is_sellable.eq(value)).execute(&mut self.connection),
            AssetUpdate::IsEnabled(value) => diesel::update(target).set(is_enabled.eq(value)).execute(&mut self.connection),
            AssetUpdate::Rank(value) => diesel::update(target).set(rank.eq(value)).execute(&mut self.connection),
            AssetUpdate::StakingApr(value) => diesel::update(target).set(staking_apr.eq(value)).execute(&mut self.connection),
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
        assets.find(asset_id).select(Asset::as_select()).first(&mut self.connection)
    }

    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<Asset>, diesel::result::Error> {
        assets.filter(id.eq_any(asset_ids)).select(Asset::as_select()).load(&mut self.connection)
    }

    fn get_assets_with_prices(&mut self, asset_ids: Vec<String>) -> Result<Vec<(Asset, Option<Price>)>, diesel::result::Error> {
        use crate::schema::prices;
        use crate::schema::prices_assets;

        assets
            .filter(id.eq_any(asset_ids))
            .left_join(prices_assets::table.on(id.eq(prices_assets::asset_id)))
            .left_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
            .select((Asset::as_select(), Option::<Price>::as_select()))
            .load(&mut self.connection)
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
