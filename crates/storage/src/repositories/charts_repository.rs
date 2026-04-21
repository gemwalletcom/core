use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::charts::{ChartResult, ChartsStore};
use primitives::{ChartPeriod, ChartTimeframe};

pub trait ChartsRepository {
    fn add_charts(&mut self, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError>;
    fn add_charts_hourly(&mut self, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError>;
    fn add_charts_daily(&mut self, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError>;
    fn get_charts(&mut self, price_id: &str, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError>;
    fn aggregate_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError>;
    fn cleanup_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError>;
}

impl ChartsRepository for DatabaseClient {
    fn add_charts(&mut self, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::add_charts(self, values)?)
    }

    fn add_charts_hourly(&mut self, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::add_charts_hourly(self, values)?)
    }

    fn add_charts_daily(&mut self, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::add_charts_daily(self, values)?)
    }

    fn get_charts(&mut self, price_id: &str, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError> {
        Ok(ChartsStore::get_charts(self, price_id, period)?)
    }

    fn aggregate_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::aggregate_charts(self, timeframe)?)
    }

    fn cleanup_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::cleanup_charts(self, timeframe)?)
    }
}
