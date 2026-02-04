use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cacher::{CacheError, CacherClient};
use job_runner::{JobError, JobSchedule, RunDecision};

pub struct CacheJobSchedule {
    cacher: CacherClient,
}

impl CacheJobSchedule {
    pub fn new(cacher: &CacherClient) -> Self {
        Self { cacher: cacher.clone() }
    }

    fn key(job_name: &str) -> String {
        format!("jobs:last_success:{}", job_name)
    }

    fn boxed<E>(error: E) -> JobError
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Box::new(error)
    }

    async fn last_success(&self, job_name: &str) -> Result<Option<SystemTime>, JobError> {
        let key = Self::key(job_name);
        match self.cacher.get_value::<u64>(&key).await {
            Ok(secs) => Ok(Some(UNIX_EPOCH + Duration::from_secs(secs))),
            Err(err) => {
                if err.downcast_ref::<CacheError>().is_some() {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }
}

#[async_trait]
impl JobSchedule for CacheJobSchedule {
    async fn evaluate(&self, job_name: &str, interval: Duration, now: SystemTime) -> Result<RunDecision, JobError> {
        if let Some(last_success) = self.last_success(job_name).await? {
            let elapsed = now.duration_since(last_success).unwrap_or_default();
            if elapsed < interval {
                return Ok(RunDecision::Wait(interval - elapsed));
            }
        }
        Ok(RunDecision::Run)
    }

    async fn mark_success(&self, job_name: &str, timestamp: SystemTime) -> Result<(), JobError> {
        let duration = timestamp.duration_since(UNIX_EPOCH).map_err(Self::boxed)?;
        self.cacher.set_value(&Self::key(job_name), &duration.as_secs()).await?;
        Ok(())
    }
}
