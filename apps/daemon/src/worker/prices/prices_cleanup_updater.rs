use cacher::{CacheKey, CacherClient};
use chrono::Utc;
use primitives::{ConfigKey, PriceProvider};
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use storage::database::prices::PriceFilter;
use storage::{Database, PricesRepository};

pub struct PricesCleanupUpdater {
    database: Database,
    cacher: CacherClient,
    config: Arc<ConfigCacher>,
    provider: PriceProvider,
}

impl PricesCleanupUpdater {
    pub fn new(database: Database, cacher: CacherClient, config: Arc<ConfigCacher>, provider: PriceProvider) -> Self {
        Self {
            database,
            cacher,
            config,
            provider,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let cutoff = (Utc::now() - chrono::Duration::from_std(self.config.get_duration(ConfigKey::PriceOutdated)?)?).naive_utc();
        let ids: Vec<String> = self
            .database
            .prices()?
            .get_prices_by_filter(vec![PriceFilter::Provider(self.provider), PriceFilter::UpdatedBefore(cutoff)])?
            .into_iter()
            .map(|p| p.id)
            .collect();
        if ids.is_empty() {
            return Ok(0);
        }
        let deleted = self.database.prices()?.delete_prices(ids.clone())?;
        self.cacher.remove_from_set_cached(CacheKey::ChartsHistory(self.provider.id()), &ids).await?;
        Ok(deleted)
    }
}
