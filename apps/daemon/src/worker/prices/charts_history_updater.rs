use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use chrono::{DateTime, Utc};
use gem_tracing::info_with_fields;
use prices::PriceAssetsProvider;
use primitives::{ChartValue, ConfigParamKey, DAY, SECONDS_PER_DAY, SECONDS_PER_HOUR};
use storage::database::prices::PriceFilter;
use storage::models::{ChartRow, PriceRow};
use storage::{ChartsRepository, ConfigCacher, Database, PricesRepository};

pub struct ChartsHistoryUpdater {
    provider: Arc<dyn PriceAssetsProvider>,
    database: Database,
    config: ConfigCacher,
    cacher: CacherClient,
}

impl ChartsHistoryUpdater {
    pub fn new(provider: Arc<dyn PriceAssetsProvider>, database: Database, cacher: CacherClient) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            provider,
            database,
            config,
            cacher,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let provider_id = provider.id();
        let hourly_duration = self.config.get_param_duration(&ConfigParamKey::PriceProviderChartsHourlyDuration(provider))?;

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
            info_with_fields!("charts history fetch started", price_id = price.id.clone());
            let daily_count = self.sync_daily_history(price).await?;
            let hourly_count = self.sync_hourly_history(price, hourly_duration).await?;
            let chart_count = self.sync_raw_history(price).await?;
            self.cacher.add_to_set_cached(CacheKey::ChartsHistory(provider_id), std::slice::from_ref(&price.id)).await?;
            info_with_fields!(
                "charts history fetch finished",
                price_id = price.id.clone(),
                daily_rows = daily_count,
                hourly_rows = hourly_count,
                chart_rows = chart_count
            );
        }

        info_with_fields!("charts history", provider = provider_id, synced = prices_to_sync.len());
        Ok(prices_to_sync.len())
    }

    async fn sync_daily_history(&self, price: &PriceRow) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.sync_history(price, HistoryKind::Daily).await
    }

    async fn sync_hourly_history(&self, price: &PriceRow, duration: Duration) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.sync_history(price, HistoryKind::Hourly(duration)).await
    }

    async fn sync_raw_history(&self, price: &PriceRow) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.sync_history(price, HistoryKind::Raw(DAY)).await
    }

    async fn sync_history(&self, price: &PriceRow, history: HistoryKind) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values = self.fetch_history(price, history).await?;
        let rows = to_chart_rows(&price.id, values, history.bucket_size_seconds());
        self.store_history(rows, history)
    }

    async fn fetch_history(&self, price: &PriceRow, history: HistoryKind) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let values = match history {
            HistoryKind::Raw(duration) => self.provider.get_charts_raw(&price.provider_price_id, duration).await,
            HistoryKind::Daily => self.provider.get_charts_daily(&price.provider_price_id).await,
            HistoryKind::Hourly(duration) => self.provider.get_charts_hourly(&price.provider_price_id, duration).await,
        };
        values.inspect_err(|error| {
            info_with_fields!(
                "charts history fetch failed",
                price_id = price.id.clone(),
                history = history.as_str(),
                error = error.to_string()
            );
        })
    }

    fn store_history(&self, rows: Vec<ChartRow>, history: HistoryKind) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if rows.is_empty() {
            return Ok(0);
        }

        let rows = match history {
            HistoryKind::Raw(_) => self.database.charts()?.add_charts(rows)?,
            HistoryKind::Daily => self.database.charts()?.add_charts_daily(rows)?,
            HistoryKind::Hourly(_) => self.database.charts()?.add_charts_hourly(rows)?,
        };
        Ok(rows)
    }
}

#[derive(Clone, Copy)]
enum HistoryKind {
    Raw(Duration),
    Daily,
    Hourly(Duration),
}

impl HistoryKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Raw(_) => "raw",
            Self::Daily => "daily",
            Self::Hourly(_) => "hourly",
        }
    }

    fn bucket_size_seconds(&self) -> Option<i64> {
        match self {
            Self::Raw(_) => None,
            Self::Daily => Some(SECONDS_PER_DAY as i64),
            Self::Hourly(_) => Some(SECONDS_PER_HOUR as i64),
        }
    }
}

fn to_chart_rows(price_id: &str, values: Vec<ChartValue>, bucket_size_seconds: Option<i64>) -> Vec<ChartRow> {
    values
        .into_iter()
        .filter_map(|value| match bucket_size_seconds {
            Some(bucket_size_seconds) => to_chart_row(price_id, value, bucket_size_seconds),
            None => to_raw_chart_row(price_id, value),
        })
        .collect()
}

fn to_chart_row(price_id: &str, value: ChartValue, bucket_size_seconds: i64) -> Option<ChartRow> {
    let bucket = i64::from(value.timestamp).div_euclid(bucket_size_seconds) * bucket_size_seconds;
    let created_at = DateTime::<Utc>::from_timestamp(bucket, 0)?.naive_utc();
    Some(ChartRow::new(price_id.to_string(), value.value as f64, created_at))
}

fn to_raw_chart_row(price_id: &str, value: ChartValue) -> Option<ChartRow> {
    let created_at = DateTime::<Utc>::from_timestamp(i64::from(value.timestamp), 0)?.naive_utc();
    Some(ChartRow::new(price_id.to_string(), value.value as f64, created_at))
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
        let hourly = to_chart_row("bitcoin", value.clone(), SECONDS_PER_HOUR as i64).unwrap();
        let daily = to_chart_row("bitcoin", value, SECONDS_PER_DAY as i64).unwrap();

        assert_eq!(hourly.coin_id, "bitcoin");
        assert_eq!(hourly.price, 123.45_f32 as f64);
        assert_eq!(hourly.created_at, DateTime::<Utc>::from_timestamp(1_713_772_800, 0).unwrap().naive_utc());
        assert_eq!(daily.coin_id, "bitcoin");
        assert_eq!(daily.price, 123.45_f32 as f64);
        assert_eq!(daily.created_at, DateTime::<Utc>::from_timestamp(1_713_744_000, 0).unwrap().naive_utc());
    }

    #[test]
    fn test_raw_chart_rows_preserve_timestamp() {
        let row = to_raw_chart_row(
            "bitcoin",
            ChartValue {
                timestamp: 1_713_774_896,
                value: 123.45,
            },
        )
        .unwrap();

        assert_eq!(row.coin_id, "bitcoin");
        assert_eq!(row.price, 123.45_f32 as f64);
        assert_eq!(row.created_at, DateTime::<Utc>::from_timestamp(1_713_774_896, 0).unwrap().naive_utc());
    }
}
