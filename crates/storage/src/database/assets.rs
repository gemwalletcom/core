use crate::schema::assets::dsl::*;

use crate::DatabaseClient;
use crate::models::{AssetRow, NewAssetRow};
use chrono::NaiveDateTime;
use diesel::{prelude::*, upsert::excluded};

#[derive(Debug, Clone)]
pub enum AssetUpdate {
    IsEnabled(bool),
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
    Rank(i32),
    StakingApr(Option<f64>),
    HasImage(bool),
    HasPrice(bool),
    Supply {
        circulating_supply: Option<f64>,
        total_supply: Option<f64>,
        max_supply: Option<f64>,
    },
}

impl AssetUpdate {
    pub fn supply(circulating: Option<f64>, total: Option<f64>, max: Option<f64>) -> Option<Self> {
        (circulating.is_some() || total.is_some() || max.is_some()).then_some(Self::Supply {
            circulating_supply: circulating,
            total_supply: total,
            max_supply: max,
        })
    }
}

#[derive(Debug, Clone)]
pub enum AssetFilter {
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
    HasImage(bool),
    HasPrice(bool),
    Chain(String),
}

pub(crate) trait AssetsStore {
    fn get_assets_all(&mut self) -> Result<Vec<AssetRow>, diesel::result::Error>;
    fn add_assets(&mut self, values: Vec<NewAssetRow>) -> Result<usize, diesel::result::Error>;
    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, diesel::result::Error>;
    fn upsert_assets(&mut self, values: Vec<NewAssetRow>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetRow>, diesel::result::Error>;
    fn get_asset(&mut self, asset_id: &str) -> Result<AssetRow, diesel::result::Error>;
    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetRow>, diesel::result::Error>;
    fn get_all_asset_ids(&mut self) -> Result<Vec<String>, diesel::result::Error>;
    fn get_asset_ids_updated_since(&mut self, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error>;
    fn get_swap_assets(&mut self) -> Result<Vec<String>, diesel::result::Error>;
    fn get_swap_assets_version(&mut self) -> Result<i32, diesel::result::Error>;
}

impl AssetsStore for DatabaseClient {
    fn get_assets_all(&mut self) -> Result<Vec<AssetRow>, diesel::result::Error> {
        assets.filter(is_enabled.eq(true)).select(AssetRow::as_select()).load(&mut self.connection)
    }
    fn add_assets(&mut self, values: Vec<NewAssetRow>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        diesel::insert_into(assets).values(values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, diesel::result::Error> {
        if asset_ids.is_empty() || updates.is_empty() {
            return Ok(0);
        }

        updates.into_iter().try_fold(0, |total, update| {
            let target = assets.filter(id.eq_any(&asset_ids));
            let updated = match update {
                AssetUpdate::IsEnabled(value) => diesel::update(target).set(is_enabled.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::IsSwappable(value) => diesel::update(target).set(is_swappable.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::IsBuyable(value) => diesel::update(target).set(is_buyable.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::IsSellable(value) => diesel::update(target).set(is_sellable.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::Rank(value) => diesel::update(target).set(rank.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::StakingApr(value) => diesel::update(target).set(staking_apr.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::HasImage(value) => diesel::update(target).set(has_image.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::HasPrice(value) => diesel::update(target).set(has_price.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::Supply {
                    circulating_supply: c,
                    total_supply: t,
                    max_supply: m,
                } => diesel::update(target)
                    .set((circulating_supply.eq(c), total_supply.eq(t), max_supply.eq(m)))
                    .execute(&mut self.connection)?,
            };
            Ok(total + updated)
        })
    }

    fn upsert_assets(&mut self, values: Vec<NewAssetRow>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(assets)
            .values(values)
            .on_conflict(id)
            .do_update()
            .set((rank.eq(excluded(rank)),))
            .execute(&mut self.connection)
    }

    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetRow>, diesel::result::Error> {
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
                AssetFilter::HasImage(value) => {
                    query = query.filter(has_image.eq(value));
                }
                AssetFilter::HasPrice(value) => {
                    query = query.filter(has_price.eq(value));
                }
                AssetFilter::Chain(value) => {
                    query = query.filter(chain.eq(value));
                }
            }
        }

        query.select(AssetRow::as_select()).load(&mut self.connection)
    }

    fn get_asset(&mut self, asset_id: &str) -> Result<AssetRow, diesel::result::Error> {
        assets.find(asset_id).select(AssetRow::as_select()).first(&mut self.connection)
    }

    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetRow>, diesel::result::Error> {
        assets.filter(id.eq_any(asset_ids)).select(AssetRow::as_select()).load(&mut self.connection)
    }

    fn get_all_asset_ids(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        assets.select(id).load(&mut self.connection)
    }

    fn get_asset_ids_updated_since(&mut self, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error> {
        assets.filter(updated_at.gt(since)).select(id).load(&mut self.connection)
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
}
