use std::time::{Duration, SystemTime};

use async_trait::async_trait;

use crate::JobError;

pub enum RunDecision {
    Run(JobContext),
    Wait(Duration),
}

#[derive(Debug, Clone)]
pub struct JobContext {
    pub last_success_at: Option<u64>,
}

#[async_trait]
pub trait JobSchedule: Send + Sync {
    async fn evaluate(&self, job_name: &str, interval: Duration, now: SystemTime) -> Result<RunDecision, JobError>;
    async fn mark_success(&self, job_name: &str, timestamp: SystemTime) -> Result<(), JobError>;
}
