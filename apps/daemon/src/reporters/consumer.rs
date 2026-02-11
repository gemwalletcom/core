use std::sync::Arc;

use async_trait::async_trait;
use streamer::ConsumerStatusReporter;

use crate::metrics::consumer::ConsumerMetrics;

pub struct ConsumerReporter {
    metrics: Arc<ConsumerMetrics>,
}

impl ConsumerReporter {
    pub fn new(metrics: Arc<ConsumerMetrics>) -> Self {
        Self { metrics }
    }
}

#[async_trait]
impl ConsumerStatusReporter for ConsumerReporter {
    async fn report_success(&self, name: &str, duration: u64, result: &str) {
        self.metrics.record_success(name, duration, result);
    }

    async fn report_error(&self, name: &str, error: &str) {
        self.metrics.record_error(name, error);
    }
}
