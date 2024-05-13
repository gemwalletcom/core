use clickhouse::{error::Result, Client};

use crate::models::{CreateChart, GetChart};

pub struct ClickhouseDatabase {
    client: Client,
}

pub const CREATE_TABLES: &str = include_str!("./clickhouse_migration.sql");
pub const CHARTS_TABLE_NAME: &str = "charts";

//TODO: Migrate to storage crate
impl ClickhouseDatabase {
    pub fn new(url: &str) -> Self {
        let client = Client::default().with_url(url).with_database("api");
        Self { client }
    }

    pub async fn migrations(&self) -> Result<()> {
        self.client.query(CREATE_TABLES).execute().await
    }

    pub async fn add_charts(&self, charts: Vec<CreateChart>) -> Result<usize> {
        let mut inserter = self
            .client
            .inserter(CHARTS_TABLE_NAME)?
            .with_max_entries(50);

        for chart in charts.clone() {
            inserter.write(&chart).await?;
        }
        inserter.end().await?;
        Ok(charts.len())
    }

    pub async fn get_charts(
        &self,
        coin_id: &str,
        period: &str,
        period_limit: i32,
    ) -> Result<Vec<GetChart>> {
        let vec = self
            .client
            .query(
                "
            SELECT
                avg(price) as price,
                toStartOfInterval(ts, INTERVAL ?) as date
            FROM
                charts
            WHERE
                coin_id = ?
                AND ts >= subtractMinutes (now(), ?)
            group BY (coin_id, date)
            ORDER BY date ASC
        ",
            )
            .bind(period)
            .bind(coin_id)
            .bind(period_limit)
            .fetch_all::<GetChart>()
            .await?;
        Ok(vec)
    }
}
