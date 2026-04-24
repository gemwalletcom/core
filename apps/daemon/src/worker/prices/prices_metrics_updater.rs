use chrono::{Duration, Utc};
use primitives::PriceProvider;
use std::collections::HashMap;
use std::error::Error;
use storage::database::prices::PriceFilter;
use storage::{ChartFilter, ChartsRepository, Database, PriceUpdate, PricesRepository};

pub struct PricesMetricsUpdater {
    database: Database,
    provider: PriceProvider,
}

impl PricesMetricsUpdater {
    pub fn new(database: Database, provider: PriceProvider) -> Self {
        Self { database, provider }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if self.provider.supports_price_change_24h() {
            return Ok(0);
        }
        let rows = self.database.prices()?.get_prices_by_filter(vec![PriceFilter::Provider(self.provider)])?;
        if rows.is_empty() {
            return Ok(0);
        }

        let now = Utc::now();
        let upper = (now - Duration::hours(24)).naive_utc();
        let lower = (now - Duration::hours(25)).naive_utc();
        let price_ids: Vec<String> = rows.iter().map(|p| p.id.clone()).collect();
        let prices_24h_ago: HashMap<String, f64> = self
            .database
            .charts()?
            .get_charts_by_filter(vec![ChartFilter::CreatedBefore(upper), ChartFilter::CreatedAfter(lower), ChartFilter::PriceIds(price_ids)])?
            .into_iter()
            .collect();

        let mut updated = 0;
        for row in rows {
            if row.price == 0.0 {
                continue;
            }
            let prev = prices_24h_ago.get(&row.id).copied().unwrap_or(0.0);
            if prev == 0.0 {
                continue;
            }
            let change = (row.price - prev) / prev * 100.0;
            updated += self
                .database
                .prices()?
                .update_prices(vec![row.id.clone()], vec![PriceUpdate::PriceChangePercentage24h(change)])?;
        }
        Ok(updated)
    }
}
