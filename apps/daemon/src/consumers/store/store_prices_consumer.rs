use async_trait::async_trait;
use pricer::PriceClient;
use primitives::AssetIdVecExt;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use storage::models::{ChartRow, PriceRow};
use storage::{AssetsRepository, ChartsRepository, Database, PricesRepository};
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
        let received_prices: Vec<PriceRow> = payload.prices.into_iter().map(PriceRow::from_price_data).collect();
        let received_price_ids: Vec<String> = received_prices.iter().map(|p| p.id.clone()).collect();

        let asset_mappings = self.database.prices()?.get_prices_assets_for_price_ids(received_price_ids)?;
        let mapped_price_ids: HashSet<String> = asset_mappings.iter().map(|m| m.price_id.clone()).collect();
        let prices_to_store: Vec<PriceRow> = received_prices.into_iter().filter(|p| mapped_price_ids.contains(&p.id)).collect();
        if prices_to_store.is_empty() {
            return Ok(0);
        }
        self.database.prices()?.set_prices(prices_to_store)?;

        let asset_ids: Vec<String> = asset_mappings.into_iter().map(|m| m.asset_id.to_string()).collect::<HashSet<_>>().into_iter().collect();

        let primary_prices = self.database.prices()?.get_primary_prices(&asset_ids)?;
        if primary_prices.is_empty() {
            return Ok(0);
        }

        let charts: Vec<ChartRow> = primary_prices.iter().map(|(_, price)| ChartRow::from_price(price.clone())).collect();
        self.database.charts()?.add_charts(charts)?;

        let count = primary_prices.len();
        let primary_asset_ids: Vec<_> = primary_prices.iter().map(|(id, _)| id.clone()).collect();
        let assets_by_id: HashMap<String, _> = self
            .database
            .assets()?
            .get_assets_rows(primary_asset_ids.ids())?
            .into_iter()
            .map(|a| (a.id.clone(), a))
            .collect();
        let cache_entries = primary_prices
            .into_iter()
            .filter_map(|(asset_id, price)| assets_by_id.get(&asset_id.to_string()).map(|asset| price.as_price_asset_info(asset)))
            .collect();
        self.price_client.set_cache_prices(cache_entries, self.ttl_seconds).await?;

        Ok(count)
    }
}
