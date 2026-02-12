use std::sync::Arc;

use async_trait::async_trait;
use job_runner::JobStatusReporter;

use crate::metrics::job::JobMetrics;

pub struct JobReporter {
    metrics: Arc<JobMetrics>,
}

impl JobReporter {
    pub fn new(metrics: Arc<JobMetrics>) -> Self {
        Self { metrics }
    }
}

#[async_trait]
impl JobStatusReporter for JobReporter {
    async fn report(&self, name: &str, interval: u64, duration: u64, success: bool, error: Option<String>) {
        self.metrics.report(name, interval, duration, success, error);
    }
}
