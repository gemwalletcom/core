use clickhouse::{error::Result, Client};

use super::model::ChartPrice;

pub struct ClickhouseDatabase {
    client: Client,
}

impl ClickhouseDatabase {
    pub fn new(url: &str) -> Self {
        let client = Client::default()
            .with_url(url)
            .with_database("api");
            
        Self {
            client,
        }
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