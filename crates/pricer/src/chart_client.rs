use primitives::{ChartPeriod, ChartValue, DEFAULT_FIAT_CURRENCY};
use std::error::Error;
use storage::{DatabaseClient, DatabaseClientExt};

pub struct ChartClient {
    database: DatabaseClient,
}

impl ChartClient {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn get_coin_id(&mut self, asset_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.database.repositories().prices().get_price(asset_id)?.id.clone())
    }

    pub async fn get_charts_prices(&mut self, coin_id: &str, period: ChartPeriod, currency: &str) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let base_rate = self.database.repositories().fiat().get_fiat_rate(DEFAULT_FIAT_CURRENCY)?;
        let rate = self.database.repositories().fiat().get_fiat_rate(currency)?;
        let rate_multiplier = rate.multiplier(base_rate.rate);

        let charts = self.database.repositories().charts().get_charts(coin_id.to_string(), &period)?;
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
