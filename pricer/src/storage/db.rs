use clickhouse::{error::Result, Client};

use super::model::ChartPrice;

pub struct ClickhouseDatabase {
    client: Client,
}

const CREATE_TABLES: &str = include_str!("./migration.sql");

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

    pub async fn add_charts(&self, charts: Vec<ChartPrice>) -> Result<u64> {
        let mut inserter = self.client
            .inserter("charts")?
            .with_max_rows(100);
        
        for chart in charts {
            inserter.write(&chart)?;
        }
        Ok(inserter.end().await?.rows)
    }
}