use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::charts::{ChartFilter, ChartResult, ChartsStore};
use chrono::NaiveDateTime;
use primitives::{ChartPeriod, ChartTimeframe};

pub trait ChartsRepository {
    fn add_charts(&mut self, timeframe: ChartTimeframe, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError>;
    fn get_charts(&mut self, price_id: &str, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError>;
    fn aggregate_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError>;
    fn delete_charts(&mut self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError>;
    fn get_charts_by_filter(&mut self, filters: Vec<ChartFilter>) -> Result<Vec<(String, f64)>, DatabaseError>;
}

impl ChartsRepository for DatabaseClient {
    fn add_charts(&mut self, timeframe: ChartTimeframe, values: Vec<crate::models::ChartRow>) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::add_charts(self, timeframe, values)?)
    }

    fn get_charts(&mut self, price_id: &str, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError> {
        Ok(ChartsStore::get_charts(self, price_id, period)?)
    }

    fn aggregate_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::aggregate_charts(self, timeframe)?)
    }

    fn delete_charts(&mut self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(ChartsStore::delete_charts(self, timeframe, before)?)
    }

    fn get_charts_by_filter(&mut self, filters: Vec<ChartFilter>) -> Result<Vec<(String, f64)>, DatabaseError> {
        Ok(ChartsStore::get_charts_by_filter(self, filters)?)
    }
}
