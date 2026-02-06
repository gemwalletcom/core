use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use job_runner::{JobError, JobSchedule, JobStatusReporter, RunDecision};
use primitives::JobStatus;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct CacherJobTracker {
    cacher: CacherClient,
    service: String,
}

impl CacherJobTracker {
    pub fn new(cacher: CacherClient, service: &str) -> Self {
        Self {
            cacher,
            service: service.to_string(),
        }
    }

    fn job_key(&self, job_name: &str) -> String {
        format!("{}:{}", self.service, job_name)
    }

    async fn load_status(&self, job_name: &str) -> JobStatus {
        let cache_key = CacheKey::JobStatus(&self.job_key(job_name));
        self.cacher.get_value(&cache_key.key()).await.unwrap_or_default()
    }

    async fn persist_status(&self, job_name: &str, status: &JobStatus) -> Result<(), JobError> {
        let cache_key = CacheKey::JobStatus(&self.job_key(job_name));
        self.cacher.set_cached(cache_key, status).await
    }
}

impl JobStatusReporter for CacherJobTracker {
    fn report(&self, name: &str, interval: u64, duration: u64, success: bool, error: Option<String>) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let cacher = self.cacher.clone();
        let job_key = self.job_key(name);
        Box::pin(async move {
            let cache_key = CacheKey::JobStatus(&job_key);
            let mut status = cacher.get_value::<JobStatus>(&cache_key.key()).await.unwrap_or_default();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

            status.interval = interval;
            status.duration = duration;

            if success {
                status.last_success = Some(timestamp);
                status.last_error = None;
                status.last_error_at = None;
            } else if let Some(msg) = error {
                status.last_error = Some(msg);
                status.last_error_at = Some(timestamp);
            }

            let _ = cacher.set_cached(cache_key, &status).await;
        })
    }
}

#[async_trait]
impl JobSchedule for CacherJobTracker {
    async fn evaluate(&self, job_name: &str, interval: Duration, now: SystemTime) -> Result<RunDecision, JobError> {
        let status = self.load_status(job_name).await;
        if let Some(last_success) = status.last_success {
            let last_success_time = UNIX_EPOCH + Duration::from_secs(last_success);
            let elapsed = now.duration_since(last_success_time).unwrap_or_default();
            if elapsed < interval {
                return Ok(RunDecision::Wait(interval - elapsed));
            }
        }
        Ok(RunDecision::Run)
    }

    async fn mark_success(&self, job_name: &str, timestamp: SystemTime) -> Result<(), JobError> {
        let mut status = self.load_status(job_name).await;
        let seconds = timestamp.duration_since(UNIX_EPOCH).map_err(|err| Box::new(err) as JobError)?.as_secs();
        status.last_success = Some(seconds);
        status.last_error = None;
        status.last_error_at = None;
        self.persist_status(job_name, &status).await
    }
}
