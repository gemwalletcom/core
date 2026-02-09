use std::time::SystemTime;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::ConsumerStatus;
use streamer::ConsumerStatusReporter;

pub struct ConsumerReporter {
    cacher: CacherClient,
}

impl ConsumerReporter {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }
}

#[async_trait]
impl ConsumerStatusReporter for ConsumerReporter {
    async fn report_success(&self, name: &str, duration: u64, result: &str) {
        let cache_key = CacheKey::ConsumerStatus(name);
        let key = cache_key.key();
        let mut status = self.cacher.get_value::<ConsumerStatus>(&key).await.unwrap_or_default();
        let timestamp = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        status.total_processed += 1;
        status.last_success = Some(timestamp);
        status.last_result = Some(result.to_string());

        let prev_total = status.total_processed - 1;
        status.avg_duration = (status.avg_duration * prev_total + duration) / status.total_processed;

        let _ = self.cacher.set_cached(cache_key, &status).await;
    }

    async fn report_error(&self, name: &str, error: &str) {
        let cache_key = CacheKey::ConsumerStatus(name);
        let key = cache_key.key();
        let mut status = self.cacher.get_value::<ConsumerStatus>(&key).await.unwrap_or_default();

        status.total_errors += 1;

        super::record_error(&mut status.errors, error);

        let _ = self.cacher.set_cached(cache_key, &status).await;
    }
}
