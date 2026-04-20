use async_trait::async_trait;
use pricer::PriceClient;
use std::collections::HashSet;
use std::error::Error;
use storage::models::{ChartRow, PriceRow};
use storage::{ChartsRepository, Database, PricesRepository};
use streamer::{PricesPayload, consumer::MessageConsumer};

pub struct StorePricesConsumer {
    pub database: Database,
    pub price_client: PriceClient,
    pub ttl_seconds: i64,
}

impl StorePricesConsumer {
    pub fn new(database: Database, price_client: PriceClient, ttl_seconds: i64) -> Self {
        Self {
            database,
            price_client,
            ttl_seconds,
        }
    }
}

#[async_trait]
impl MessageConsumer<PricesPayload, usize> for StorePricesConsumer {
    async fn should_process(&self, _payload: PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: PricesPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let rows: Vec<PriceRow> = payload.prices.into_iter().map(PriceRow::from_price_data).collect();
        let price_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        self.database.prices()?.set_prices(rows)?;

        let asset_ids: Vec<String> = self
            .database
            .prices()?
            .get_prices_assets_for_price_ids(price_ids)?
            .into_iter()
            .map(|x| x.asset_id.to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let primary = self.database.prices()?.get_primary_prices(&asset_ids)?;
        if primary.is_empty() {
            return Ok(0);
        }

        let charts: Vec<ChartRow> = primary.iter().map(|(_, row)| ChartRow::from_price(row.clone())).collect();
        self.database.charts()?.add_charts(charts)?;

        let count = primary.len();
        let infos = primary.into_iter().map(|(id, row)| row.as_price_asset_info(id)).collect();
        self.price_client.set_cache_prices(infos, self.ttl_seconds).await?;

        Ok(count)
    }
}
