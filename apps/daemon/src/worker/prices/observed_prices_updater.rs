use std::error::Error;

use cacher::{CacheKey, CacherClient};
use pricer::PriceClient;
use streamer::{FetchPricesPayload, StreamProducer, StreamProducerQueue};

pub struct ObservedPricesUpdater {
    cacher_client: CacherClient,
    price_client: PriceClient,
    stream_producer: StreamProducer,
    max_assets: usize,
    min_observers: usize,
}

impl ObservedPricesUpdater {
    pub fn new(cacher_client: CacherClient, price_client: PriceClient, stream_producer: StreamProducer, max_assets: usize, min_observers: usize) -> Self {
        Self {
            cacher_client,
            price_client,
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

        let price_ids = self.price_client.get_price_ids_for_asset_ids(&asset_ids)?;
        if price_ids.is_empty() {
            return Ok(0);
        }

        let count = price_ids.len();
        let payload = FetchPricesPayload::new(price_ids);
        self.stream_producer.publish_fetch_prices(payload).await?;
        Ok(count)
    }

    async fn get_observed_assets(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let key = CacheKey::ObservedAssets;
        self.cacher_client
            .sorted_set_range_by_score(&key.key(), self.min_observers as f64, f64::INFINITY, self.max_assets)
            .await
    }
}
