use std::time::{Duration, SystemTime};

use async_trait::async_trait;

use crate::JobError;

pub enum RunDecision {
    Run,
    Wait(Duration),
}

#[async_trait]
pub trait JobSchedule: Send + Sync {
    async fn evaluate(&self, job_name: &str, interval: Duration, now: SystemTime) -> Result<RunDecision, JobError>;
    async fn mark_success(&self, job_name: &str, timestamp: SystemTime) -> Result<(), JobError>;
}

#[derive(Default)]
pub struct RunAlways;

#[async_trait]
impl JobSchedule for RunAlways {
    async fn evaluate(&self, _job_name: &str, _interval: Duration, _now: SystemTime) -> Result<RunDecision, JobError> {
        Ok(RunDecision::Run)
    }

    async fn mark_success(&self, _job_name: &str, _timestamp: SystemTime) -> Result<(), JobError> {
        Ok(())
    }
}
