use crate::DatabaseClient;
use crate::DatabaseError;
use crate::database::assets_usage_ranks::AssetsUsageRanksStore;
use crate::models::AssetUsageRankRow;
use chrono::NaiveDateTime;

pub trait AssetsUsageRanksRepository {
    fn upsert_usage_ranks(&mut self, values: Vec<AssetUsageRankRow>) -> Result<usize, DatabaseError>;
    fn delete_usage_ranks_before(&mut self, before: NaiveDateTime) -> Result<usize, DatabaseError>;
    fn get_all_usage_ranks(&mut self) -> Result<Vec<AssetUsageRankRow>, DatabaseError>;
}

impl AssetsUsageRanksRepository for DatabaseClient {
    fn upsert_usage_ranks(&mut self, values: Vec<AssetUsageRankRow>) -> Result<usize, DatabaseError> {
        Ok(AssetsUsageRanksStore::upsert_usage_ranks(self, values)?)
    }

    fn delete_usage_ranks_before(&mut self, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(AssetsUsageRanksStore::delete_usage_ranks_before(self, before)?)
    }

    fn get_all_usage_ranks(&mut self) -> Result<Vec<AssetUsageRankRow>, DatabaseError> {
        Ok(AssetsUsageRanksStore::get_all_usage_ranks(self)?)
    }
}
