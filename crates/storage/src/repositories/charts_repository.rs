use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::charts::{ChartResult, ChartsStore};
use primitives::ChartPeriod;

pub trait ChartsRepository {
    fn add_charts(&mut self, values: Vec<crate::models::Chart>) -> Result<usize, DatabaseError>;
    fn get_charts(&mut self, target_coin_id: String, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError>;
    fn aggregate_hourly_charts(&mut self) -> Result<usize, DatabaseError>;
    fn aggregate_daily_charts(&mut self) -> Result<usize, DatabaseError>;
    fn cleanup_charts_data(&mut self) -> Result<usize, DatabaseError>;
}

impl ChartsRepository for DatabaseClient {
    fn add_charts(&mut self, values: Vec<crate::models::Chart>) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::add_charts(self, values)?)
    }

    fn get_charts(&mut self, target_coin_id: String, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError> {
        Ok(ChartsStore::get_charts(self, target_coin_id, period)?)
    }

    fn aggregate_hourly_charts(&mut self) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::aggregate_hourly_charts(self)?)
    }

    fn aggregate_daily_charts(&mut self) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::aggregate_daily_charts(self)?)
    }

    fn cleanup_charts_data(&mut self) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::cleanup_charts_data(self)?)
    }
}
