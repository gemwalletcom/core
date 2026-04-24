use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use prices::AssetPriceMapping;
use primitives::{AssetId, PriceProvider};
use storage::{Database, PricesRepository};
use streamer::StreamProducer;

use crate::worker::prices::{AssetsProviders, prices_updater::PricesUpdater};

#[derive(Clone, Copy)]
pub struct ObservedPricesConfig {
    pub max_assets: usize,
    pub min_observers: usize,
    pub primary_price_max_age: Duration,
}

pub struct ObservedPricesUpdater {
    cacher_client: CacherClient,
    database: Database,
    providers: AssetsProviders,
    stream_producer: StreamProducer,
    config: ObservedPricesConfig,
}

impl ObservedPricesUpdater {
    pub fn new(cacher_client: CacherClient, database: Database, providers: AssetsProviders, stream_producer: StreamProducer, config: ObservedPricesConfig) -> Self {
        Self {
            cacher_client,
            database,
            providers,
            stream_producer,
            config,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let asset_ids: Vec<AssetId> = self.get_observed_assets().await?.into_iter().filter_map(|id| AssetId::new(&id)).collect();
        if asset_ids.is_empty() {
            return Ok(0);
        }

        let mut by_provider: HashMap<PriceProvider, Vec<AssetPriceMapping>> = HashMap::new();
        for (asset_id, row) in self.database.prices()?.get_primary_prices(&asset_ids, self.config.primary_price_max_age)? {
            by_provider.entry(row.provider.0).or_default().push(AssetPriceMapping::new(asset_id, row.provider_price_id));
        }

        let mut total = 0;
        for (provider, mappings) in by_provider {
            let Some(instance) = self.providers.get(&provider).cloned() else {
                continue;
            };
            total += PricesUpdater::new(instance, self.database.clone(), self.stream_producer.clone())
                .update_prices(mappings)
                .await?;
        }
        Ok(total)
    }

    async fn get_observed_assets(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let key = CacheKey::ObservedAssets;
        self.cacher_client
            .sorted_set_range_by_score(&key.key(), self.config.min_observers as f64, f64::INFINITY, self.config.max_assets)
            .await
    }
}
