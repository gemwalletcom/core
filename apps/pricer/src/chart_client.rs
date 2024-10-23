use primitives::{ChartPeriod, ChartValue, DEFAULT_FIAT_CURRENCY};
use std::error::Error;
use storage::{models::CreateChart, ClickhouseDatabase, DatabaseClient};

pub struct ChartClient {
    database: DatabaseClient,
    clickhouse_database: ClickhouseDatabase,
}

impl ChartClient {
    pub fn new(database_url: &str, clichouse_database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        let clickhouse_database = ClickhouseDatabase::new(clichouse_database_url);
        Self { database, clickhouse_database }
    }

    pub fn get_coin_id(&mut self, asset_id: &str) -> Result<String, Box<dyn Error>> {
        Ok(self.database.get_price(asset_id)?.id.clone())
    }

    pub async fn set_charts(&mut self, charts: Vec<CreateChart>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.clickhouse_database.add_charts(charts.clone()).await?;
        Ok(charts.len())
    }
    pub async fn get_charts_prices(&mut self, coin_id: &str, period: ChartPeriod, currency: &str) -> Result<Vec<ChartValue>, Box<dyn Error>> {
        let base_rate = self.database.get_fiat_rate(DEFAULT_FIAT_CURRENCY)?;
        let rate = self.database.get_fiat_rate(currency)?;
        let rate_multiplier = rate.rate / base_rate.rate;
        let interval = self.period_sql(period.clone());

        match period {
            ChartPeriod::Hour | ChartPeriod::Day | ChartPeriod::Week | ChartPeriod::Month | ChartPeriod::Quarter | ChartPeriod::Year | ChartPeriod::All => {
                let prices = self
                    .clickhouse_database
                    .get_charts(coin_id, interval, period.minutes())
                    .await?
                    .into_iter()
                    .map(|x| ChartValue {
                        timestamp: x.date,
                        value: (x.price * rate_multiplier) as f32,
                    })
                    .collect();
                Ok(prices)
            }
        }
    }

    fn period_sql(&self, period: ChartPeriod) -> &str {
        match period {
            ChartPeriod::Hour => "1 minute",
            ChartPeriod::Day => "15 minute",
            ChartPeriod::Week => "1 hour",
            ChartPeriod::Month => "6 hour",
            ChartPeriod::Quarter => "1 day",
            ChartPeriod::Year => "3 day",
            ChartPeriod::All => "3 day",
        }
    }
}
