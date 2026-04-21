use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use chrono::{DateTime, Utc};
use gem_tracing::info_with_fields;
use prices::PriceAssetsProvider;
use primitives::ChartValue;
use storage::database::prices::PriceFilter;
use storage::models::{ChartRow, PriceRow};
use storage::{ChartsRepository, Database, PricesRepository};

pub struct ChartsHistoryUpdater {
    provider: Arc<dyn PriceAssetsProvider>,
    database: Database,
    cacher: CacherClient,
}

impl ChartsHistoryUpdater {
    pub fn new(provider: Arc<dyn PriceAssetsProvider>, database: Database, cacher: CacherClient) -> Self {
        Self { provider, database, cacher }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let provider_id = provider.id();

        let synced_price_ids: HashSet<String> = self
            .cacher
            .get_set_members_cached(vec![CacheKey::ChartsHistory(provider_id).key()])
            .await?
            .into_iter()
            .collect();
        let prices_to_sync: Vec<PriceRow> = self
            .database
            .prices()?
            .get_prices_by_filter(vec![PriceFilter::Provider(provider)])?
            .into_iter()
            .filter(|p| !synced_price_ids.contains(&p.id))
            .collect();

        for price in &prices_to_sync {
            let chart_values = self.provider.get_charts_daily(&price.provider_price_id).await?;
            let chart_rows: Vec<ChartRow> = chart_values.into_iter().filter_map(|value| to_daily_row(&price.id, value)).collect();
            if !chart_rows.is_empty() {
                self.database.charts()?.add_charts_daily(chart_rows)?;
            }
            self.cacher.add_to_set_cached(CacheKey::ChartsHistory(provider_id), &[price.id.clone()]).await?;
        }

        info_with_fields!("charts history", provider = provider_id, synced = prices_to_sync.len());
        Ok(prices_to_sync.len())
    }
}

fn to_daily_row(price_id: &str, value: ChartValue) -> Option<ChartRow> {
    let bucket = DateTime::<Utc>::from_timestamp(value.timestamp as i64, 0)?.date_naive().and_hms_opt(0, 0, 0)?;
    Some(ChartRow::new(price_id.to_string(), value.value as f64, bucket))
}
