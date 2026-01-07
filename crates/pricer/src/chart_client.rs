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

    pub fn get_coin_id(&self, asset_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.database.prices()?.get_coin_id(asset_id)?)
    }

    pub async fn get_charts_prices(&self, coin_id: &str, period: ChartPeriod, currency: &str) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let base_rate = self.database.fiat()?.get_fiat_rate(DEFAULT_FIAT_CURRENCY)?.as_primitive();
        let rate = self.database.fiat()?.get_fiat_rate(currency)?.as_primitive();
        let rate_multiplier = rate.multiplier(base_rate.rate);

        let charts = self.database.charts()?.get_charts(coin_id.to_string(), &period)?;
        let prices = charts
            .into_iter()
            .map(|x| ChartValue {
                timestamp: x.0.and_utc().timestamp() as i32,
                value: (x.1 * rate_multiplier) as f32,
            })
            .collect();
        Ok(prices)
    }
}
