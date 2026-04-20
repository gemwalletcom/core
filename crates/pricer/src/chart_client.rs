use primitives::{ChartPeriod, ChartValue, DEFAULT_FIAT_CURRENCY};
use std::error::Error;
use storage::{ChartsRepository, Database, PricesRepository};

#[derive(Clone)]
pub struct ChartClient {
    database: Database,
}

impl ChartClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_charts_prices(&self, asset_id: &str, period: ChartPeriod, currency: &str) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let base_rate = self.database.fiat()?.get_fiat_rate(DEFAULT_FIAT_CURRENCY)?.as_primitive();
        let rate = self.database.fiat()?.get_fiat_rate(currency)?.as_primitive();
        let rate_multiplier = rate.multiplier(base_rate.rate);

        let key = self.database.prices()?.get_primary_price_key(asset_id)?;
        Ok(self
            .database
            .charts()?
            .get_charts(&key.id(), &period)?
            .into_iter()
            .map(|(ts, price)| ChartValue {
                timestamp: ts.and_utc().timestamp() as i32,
                value: (price * rate_multiplier) as f32,
            })
            .collect())
    }
}
