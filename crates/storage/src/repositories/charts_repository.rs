use std::error::Error;

use crate::database::charts::{ChartResult, ChartsStore};
use crate::DatabaseClient;
use primitives::ChartPeriod;

pub trait ChartsRepository {
    fn add_charts(&mut self, values: Vec<crate::models::Chart>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_charts(&mut self, target_coin_id: String, period: &ChartPeriod) -> Result<Vec<ChartResult>, Box<dyn Error + Send + Sync>>;
    fn aggregate_hourly_charts(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn aggregate_daily_charts(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn cleanup_charts_data(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl ChartsRepository for DatabaseClient {
    fn add_charts(&mut self, values: Vec<crate::models::Chart>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ChartsStore::add_charts(self, values)?)
    }

    fn get_charts(&mut self, target_coin_id: String, period: &ChartPeriod) -> Result<Vec<ChartResult>, Box<dyn Error + Send + Sync>> {
        Ok(ChartsStore::get_charts(self, target_coin_id, period)?)
    }

    fn aggregate_hourly_charts(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ChartsStore::aggregate_hourly_charts(self)?)
    }

    fn aggregate_daily_charts(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ChartsStore::aggregate_daily_charts(self)?)
    }

    fn cleanup_charts_data(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ChartsStore::cleanup_charts_data(self)?)
    }
}
