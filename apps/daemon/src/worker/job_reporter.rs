use std::future::Future;
use std::pin::Pin;

use cacher::{CacheKey, CacherClient};
use gem_tracing::info_with_fields;
use job_runner::JobStatusReporter;
use primitives::JobStatus;

pub struct CacherJobReporter {
    cacher: CacherClient,
    service: String,
}

impl CacherJobReporter {
    pub fn new(cacher: CacherClient, service: &str) -> Self {
        Self {
            cacher,
            service: service.to_string(),
        }
    }
}

pub fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace(' ', "_")
}

impl JobStatusReporter for CacherJobReporter {
    fn report(&self, name: &str, interval: u64, duration: u64, success: bool, error: Option<String>) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let normalized = normalize_name(&format!("{}:{}", self.service, name));
        Box::pin(async move {
            let cache_key = CacheKey::JobStatus(&normalized);
            let key = cache_key.key();
            let mut status = self.cacher.get_value::<JobStatus>(&key).await.unwrap_or_default();
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

            status.interval = interval;
            status.duration = duration;

            if success {
                status.last_success = Some(timestamp);
            } else if let Some(msg) = error {
                status.last_error = Some(msg);
                status.last_error_at = Some(timestamp);
            }

            if let Err(e) = self.cacher.set_cached(cache_key, &status).await {
                info_with_fields!("job status report failed", job = key, error = format!("{:?}", e));
            }
        })
    }
}
