use clickhouse::{error::Result, Client};

use crate::models::{ChartCoinPrice, ChartPrice};

pub struct ClickhouseDatabase {
    client: Client,
}

const CREATE_TABLES: &str = include_str!("./clickhouse_migration.sql");

//TODO: Migrate to storage crate
impl ClickhouseDatabase {
    pub fn new(url: &str) -> Self {
        let client = Client::default()
            .with_url(url)
            .with_database("api")
            .with_option("max_partitions_per_insert_block", "10000");
            
        Self {
            client,
        }
    }

    pub async fn migrations(&self) -> Result<()> {
        self.client.query(CREATE_TABLES).execute().await
    }

    pub async fn add_charts(&self, charts: Vec<ChartCoinPrice>) -> Result<u64> {
        let mut inserter = self.client
            .inserter("charts")?
            .with_max_rows(100);
        
        for chart in charts {
            inserter.write(&chart)?;
        }
        Ok(inserter.end().await?.rows)
    }

    pub async fn get_charts(&self, coin_id: &str, period: &str, period_limit: i32) -> Result<Vec<ChartPrice>> {
        let vec = self.client
            .query("
                SELECT
                    avg(price),
                    toStartOfInterval(created_at, INTERVAL ?) as date
                FROM
                    charts
                WHERE
                    coin_id = ?
                    AND created_at >= subtractMinutes (now(), ?)
                group BY (coin_id, date)
                ORDER BY date DESC
            ")
            .bind(period)
            .bind(coin_id)
            .bind(period_limit)
            .fetch_all::<ChartPrice>()
            .await?;
        Ok(vec)
    }

}