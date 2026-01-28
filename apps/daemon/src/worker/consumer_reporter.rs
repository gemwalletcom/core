use std::future::Future;
use std::pin::Pin;

use cacher::{CacheKey, CacherClient};
use gem_tracing::info_with_fields;
use primitives::{ConsumerError, ConsumerStatus};
use streamer::ConsumerStatusReporter;

pub struct CacherConsumerReporter {
    cacher: CacherClient,
}

impl CacherConsumerReporter {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }
}

impl ConsumerStatusReporter for CacherConsumerReporter {
    fn report_success(&self, name: &str, duration: u64, result: &str) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let normalized = name.to_string();
        let result = result.to_string();
        Box::pin(async move {
            let cache_key = CacheKey::ConsumerStatus(&normalized);
            let key = cache_key.key();
            let mut status = self.cacher.get_value::<ConsumerStatus>(&key).await.unwrap_or_default();
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

            status.total_processed += 1;
            status.last_success = Some(timestamp);
            status.last_result = Some(result);

            let prev_total = status.total_processed - 1;
            status.avg_duration = (status.avg_duration * prev_total + duration) / status.total_processed;

            if let Err(e) = self.cacher.set_cached(cache_key, &status).await {
                info_with_fields!("consumer status report failed", consumer = key, error = format!("{:?}", e));
            }
        })
    }

    fn report_error(&self, name: &str, error: &str) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let normalized = name.to_string();
        let error = error.to_string();
        Box::pin(async move {
            let cache_key = CacheKey::ConsumerStatus(&normalized);
            let key = cache_key.key();
            let mut status = self.cacher.get_value::<ConsumerStatus>(&key).await.unwrap_or_default();
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

            status.total_errors += 1;

            let truncated = if error.len() > 200 { error[..200].to_string() } else { error.clone() };

            if let Some(entry) = status.errors.iter_mut().find(|e| e.message == truncated) {
                entry.count += 1;
                entry.last_seen = timestamp;
            } else {
                status.errors.push(ConsumerError {
                    message: truncated,
                    count: 1,
                    last_seen: timestamp,
                });
            }

            if let Err(e) = self.cacher.set_cached(cache_key, &status).await {
                info_with_fields!("consumer status report failed", consumer = key, error = format!("{:?}", e));
            }
        })
    }
}
