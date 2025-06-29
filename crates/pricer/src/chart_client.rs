use primitives::{ChartPeriod, ChartValue, DEFAULT_FIAT_CURRENCY};
use std::error::Error;
use storage::{DatabaseClient, database::charts::{ChartGranularity, get_charts}};
use chrono::{DateTime, NaiveDateTime, Duration, Utc};

pub struct ChartClient {
    database: DatabaseClient,
}

impl ChartClient {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn get_coin_id(&mut self, asset_id: &str) -> Result<String, Box<dyn Error>> {
        Ok(self.database.get_price(asset_id)?.id.clone())
    }


    pub async fn get_charts_prices(&mut self, coin_id: &str, period: ChartPeriod, currency: &str) -> Result<Vec<ChartValue>, Box<dyn Error>> {
        let base_rate = self.database.get_fiat_rate(DEFAULT_FIAT_CURRENCY)?;
        let rate = self.database.get_fiat_rate(currency)?.as_primitive();
        let rate_multiplier = rate.multiplier(base_rate.rate);

        let (start_time, end_time) = Self::get_time_range_for_period(&period);
        let charts = get_charts(self.database.get_connection(), coin_id.to_string(), ChartGranularity::Minute, start_time, end_time).await?;
        let prices = charts
            .into_iter()
            .map(|x| ChartValue {
                timestamp: x.ts.and_utc().timestamp() as i32,
                value: ((x.price as f64) * rate_multiplier) as f32,
            })
            .collect();
        Ok(prices)
    }

    fn get_time_range_for_period(period: &ChartPeriod) -> (NaiveDateTime, NaiveDateTime) {
        let now = Utc::now().naive_utc();
        let (start_time, end_time) = match period {
            ChartPeriod::Hour => (now - Duration::hours(1), now),
            ChartPeriod::Day => (now - Duration::days(1), now),
            ChartPeriod::Week => (now - Duration::weeks(1), now),
            ChartPeriod::Month => (now - Duration::days(30), now),
            ChartPeriod::Quarter => (now - Duration::days(90), now),
            ChartPeriod::Year => (now - Duration::days(365), now),
            ChartPeriod::All => (DateTime::from_timestamp(0, 0).unwrap().naive_utc(), now),
        };
        (start_time, end_time)
    }
}
