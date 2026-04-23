use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use chrono::{DateTime, Utc};
use gem_tracing::info_with_fields;
use pricer::PriceClient;
use prices::PriceAssetsProvider;
use primitives::{ChartTimeframe, ChartValue, SECONDS_PER_DAY, SECONDS_PER_HOUR};
use storage::database::prices::PriceFilter;
use storage::models::{ChartRow, PriceRow};
use storage::{ChartsRepository, Database, PricesRepository};

#[derive(Clone)]
pub struct ChartsUpdater {
    prices_client: PriceClient,
}

impl ChartsUpdater {
    pub fn new(prices_client: PriceClient) -> Self {
        Self { prices_client }
    }

    pub async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.prices_client.aggregate_charts(timeframe).await
    }

    pub async fn delete_charts(&self, timeframe: ChartTimeframe, before: chrono::NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.prices_client.delete_charts(timeframe, before).await
    }
}

#[derive(Clone, Copy)]
pub struct ChartsHistoryConfig {
    pub hourly_duration: Duration,
}

pub struct ChartsHistoryUpdater {
    provider: Arc<dyn PriceAssetsProvider>,
    database: Database,
    cacher: CacherClient,
    config: ChartsHistoryConfig,
}

impl ChartsHistoryUpdater {
    pub fn new(provider: Arc<dyn PriceAssetsProvider>, database: Database, cacher: CacherClient, config: ChartsHistoryConfig) -> Self {
        Self {
            provider,
            database,
            cacher,
            config,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let provider_id = provider.id();

        let synced: HashSet<String> = self
            .cacher
            .get_set_members_cached(vec![CacheKey::ChartsHistory(provider_id).key()])
            .await?
            .into_iter()
            .collect();
        let prices: Vec<PriceRow> = self
            .database
            .prices()?
            .get_prices_by_filter(vec![PriceFilter::Provider(provider)])?
            .into_iter()
            .filter(|p| !synced.contains(&p.id))
            .collect();

        for price in &prices {
            info_with_fields!("charts history sync started", price_id = price.id.clone());
            let daily = self.sync(price, "daily", ChartTimeframe::Daily, SECONDS_PER_DAY as i64, self.provider.get_charts_daily(&price.provider_price_id)).await?;
            let hourly = self
                .sync(
                    price,
                    "hourly",
                    ChartTimeframe::Hourly,
                    SECONDS_PER_HOUR as i64,
                    self.provider.get_charts_hourly(&price.provider_price_id, self.config.hourly_duration),
                )
                .await?;
            let has_history = daily.received + hourly.received > 0;
            let extremes_updates = if has_history { self.database.prices()?.update_extremes_for_price(&price.id)? } else { 0 };
            if has_history {
                self.cacher.add_to_set_cached(CacheKey::ChartsHistory(provider_id), std::slice::from_ref(&price.id)).await?;
            }
            info_with_fields!(
                "charts history sync finished",
                price_id = price.id.clone(),
                daily_received = daily.received,
                daily_inserted = daily.inserted,
                hourly_received = hourly.received,
                hourly_inserted = hourly.inserted,
                extremes_updates = extremes_updates,
                marked_synced = has_history
            );
        }

        info_with_fields!("charts history", provider = provider_id, synced = prices.len());
        Ok(prices.len())
    }

    async fn sync(
        &self,
        price: &PriceRow,
        label: &'static str,
        timeframe: ChartTimeframe,
        bucket_size_seconds: i64,
        future: impl std::future::Future<Output = Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>>>,
    ) -> Result<HistorySyncStats, Box<dyn Error + Send + Sync>> {
        let values = future.await.inspect_err(|error| {
            info_with_fields!("charts history get failed", price_id = price.id.clone(), kind = label, error = error.to_string());
        })?;
        let rows = bucketed_chart_rows(&price.id, &values, bucket_size_seconds);
        Ok(HistorySyncStats {
            received: values.len(),
            inserted: self.database.charts()?.add_charts(timeframe, rows)?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct HistorySyncStats {
    received: usize,
    inserted: usize,
}

fn bucketed_chart_rows(price_id: &str, values: &[ChartValue], bucket_size_seconds: i64) -> Vec<ChartRow> {
    values
        .iter()
        .filter_map(|value| {
            let bucket = i64::from(value.timestamp).div_euclid(bucket_size_seconds) * bucket_size_seconds;
            let created_at = DateTime::<Utc>::from_timestamp(bucket, 0)?.naive_utc();
            Some(ChartRow::new(price_id.to_string(), value.value as f64, created_at))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_rows_are_bucketed() {
        let value = ChartValue {
            timestamp: 1_713_774_896,
            value: 123.45,
        };
        let hourly = bucketed_chart_rows("bitcoin", &[value.clone()], SECONDS_PER_HOUR as i64).remove(0);
        let daily = bucketed_chart_rows("bitcoin", &[value], SECONDS_PER_DAY as i64).remove(0);

        assert_eq!(hourly.coin_id, "bitcoin");
        assert_eq!(hourly.price, 123.45_f32 as f64);
        assert_eq!(hourly.created_at, DateTime::<Utc>::from_timestamp(1_713_772_800, 0).unwrap().naive_utc());
        assert_eq!(daily.coin_id, "bitcoin");
        assert_eq!(daily.price, 123.45_f32 as f64);
        assert_eq!(daily.created_at, DateTime::<Utc>::from_timestamp(1_713_744_000, 0).unwrap().naive_utc());
    }
}
