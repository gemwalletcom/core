use async_trait::async_trait;
use cacher::CacherClient;
use job_runner::{JobError, JobSchedule, RunDecision};
use primitives::JobStatus;
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

    async fn load_status(&self, job_name: &str) -> Option<JobStatus> {
        let cache_key = cacher::CacheKey::JobStatus(&self.job_key(job_name));
        self.cacher.get_value(&cache_key.key()).await.ok()
    }

    async fn persist_status(&self, job_name: &str, status: &JobStatus) -> Result<(), JobError> {
        let cache_key = cacher::CacheKey::JobStatus(&self.job_key(job_name));
        self.cacher.set_cached(cache_key, status).await
    }
}

#[async_trait]
impl JobSchedule for CacherJobTracker {
    async fn evaluate(&self, job_name: &str, interval: Duration, now: SystemTime) -> Result<RunDecision, JobError> {
        let Some(status) = self.load_status(job_name).await else {
            return Ok(RunDecision::Run);
        };
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
        let mut status = self.load_status(job_name).await.unwrap_or_default();
        let seconds = timestamp.duration_since(UNIX_EPOCH).map_err(|err| Box::new(err) as JobError)?.as_secs();
        status.last_success = Some(seconds);
        self.persist_status(job_name, &status).await
    }
}
