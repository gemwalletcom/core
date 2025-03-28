use crate::models::{CreateChart, GetChart};
use clickhouse::{error::Result, Client, Row};
use serde::Serialize;

pub struct ClickhouseClient {
    client: Client,
}

pub const CREATE_TABLES: &str = include_str!("./clickhouse_migration.sql");
pub const CHARTS_TABLE_NAME: &str = "charts";

impl ClickhouseClient {
    pub fn new(url: &str, database: &str) -> Self {
        let client = Client::default().with_url(url).with_database(database);
        Self { client }
    }

    pub async fn migrations(&self) -> Result<()> {
        self.client.query(CREATE_TABLES).execute().await
    }

    pub async fn add_charts(&self, charts: Vec<CreateChart>) -> Result<usize> {
        self.add_items(CHARTS_TABLE_NAME, charts).await
    }

    pub async fn add_items<T: Serialize + Clone + Row>(&self, table_name: &str, values: Vec<T>) -> Result<usize> {
        let mut inserter = self.client.insert(table_name)?.with_option("max_insert_block_size", "50");
        for value in values.clone() {
            inserter.write(&value).await?;
        }
        inserter.end().await?;
        Ok(values.len())
    }

    pub async fn get_charts(&self, coin_id: &str, period: &str, period_limit: i32) -> Result<Vec<GetChart>> {
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
