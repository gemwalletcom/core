use crate::schema::assets_usage_ranks::dsl::*;
use crate::{DatabaseClient, models::AssetUsageRankRow};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::upsert::excluded;

pub(crate) trait AssetsUsageRanksStore {
    fn upsert_usage_ranks(&mut self, values: Vec<AssetUsageRankRow>) -> Result<usize, diesel::result::Error>;
    fn delete_usage_ranks_before(&mut self, before: NaiveDateTime) -> Result<usize, diesel::result::Error>;
    fn get_all_usage_ranks(&mut self) -> Result<Vec<AssetUsageRankRow>, diesel::result::Error>;
}

impl AssetsUsageRanksStore for DatabaseClient {
    fn upsert_usage_ranks(&mut self, values: Vec<AssetUsageRankRow>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        diesel::insert_into(assets_usage_ranks)
            .values(&values)
            .on_conflict(asset_id)
            .do_update()
            .set(usage_rank.eq(excluded(usage_rank)))
            .execute(&mut self.connection)
    }

    fn delete_usage_ranks_before(&mut self, before: NaiveDateTime) -> Result<usize, diesel::result::Error> {
        diesel::delete(assets_usage_ranks.filter(updated_at.lt(before))).execute(&mut self.connection)
    }

    fn get_all_usage_ranks(&mut self) -> Result<Vec<AssetUsageRankRow>, diesel::result::Error> {
        assets_usage_ranks.select(AssetUsageRankRow::as_select()).load(&mut self.connection)
    }
}
