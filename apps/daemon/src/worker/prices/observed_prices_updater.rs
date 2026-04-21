use std::collections::HashMap;
use std::error::Error;

use cacher::{CacheKey, CacherClient};
use prices::AssetPriceMapping;
use primitives::PriceProvider;
use storage::{Database, PricesRepository};
use streamer::StreamProducer;

use crate::worker::prices::{Providers, prices_updater::PricesUpdater};

pub struct ObservedPricesUpdater {
    cacher_client: CacherClient,
    database: Database,
    providers: Providers,
    stream_producer: StreamProducer,
    max_assets: usize,
    min_observers: usize,
}

impl ObservedPricesUpdater {
    pub fn new(cacher_client: CacherClient, database: Database, providers: Providers, stream_producer: StreamProducer, max_assets: usize, min_observers: usize) -> Self {
        Self {
            cacher_client,
            database,
            providers,
            stream_producer,
            max_assets,
            min_observers,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let asset_ids = self.get_observed_assets().await?;
        if asset_ids.is_empty() {
            return Ok(0);
        }

        let mut by_provider: HashMap<PriceProvider, Vec<AssetPriceMapping>> = HashMap::new();
        for (asset_id, row) in self.database.prices()?.get_primary_prices(&asset_ids)? {
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
            .sorted_set_range_by_score(&key.key(), self.min_observers as f64, f64::INFINITY, self.max_assets)
            .await
    }
}
