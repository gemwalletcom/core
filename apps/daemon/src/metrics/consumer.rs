use std::collections::HashMap;
use std::sync::Mutex;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ConsumerLabels {
    consumer: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ConsumerErrorLabels {
    consumer: String,
    error: String,
}

#[derive(Default)]
struct ConsumerState {
    total_processed: u64,
    last_success: Option<u64>,
    avg_duration: u64,
    errors: HashMap<String, u64>,
}

pub struct ConsumerMetrics {
    consumers: Mutex<HashMap<String, ConsumerState>>,
}

impl ConsumerMetrics {
    pub fn new() -> Self {
        Self {
            consumers: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_success(&self, name: &str, duration: u64, _result: &str) {
        let mut consumers = self.consumers.lock().unwrap();
        let state = consumers.entry(name.to_string()).or_default();
        let timestamp = super::now_unix();

        state.total_processed += 1;
        state.last_success = Some(timestamp);

        let prev_total = state.total_processed - 1;
        state.avg_duration = (state.avg_duration * prev_total + duration) / state.total_processed;
    }

    pub fn record_error(&self, name: &str, error: &str) {
        let mut consumers = self.consumers.lock().unwrap();
        let state = consumers.entry(name.to_string()).or_default();
        *state.errors.entry(super::sanitize_error_message(error)).or_default() += 1;
    }
}

impl MetricsProvider for ConsumerMetrics {
    fn register(&self, registry: &mut Registry) {
        let processed = Family::<ConsumerLabels, Gauge>::default();
        let last_success_at = Family::<ConsumerLabels, Gauge>::default();
        let avg_duration = Family::<ConsumerLabels, Gauge>::default();
        let errors = Family::<ConsumerErrorLabels, Gauge>::default();

        let consumers = self.consumers.lock().unwrap();
        for (name, state) in consumers.iter() {
            let labels = ConsumerLabels { consumer: name.clone() };

            processed.get_or_create(&labels).set(state.total_processed as i64);
            if let Some(ts) = state.last_success {
                last_success_at.get_or_create(&labels).set(ts as i64);
            }
            avg_duration.get_or_create(&labels).set(state.avg_duration as i64);

            for (error, count) in &state.errors {
                let error_labels = ConsumerErrorLabels {
                    consumer: name.clone(),
                    error: error.clone(),
                };
                errors.get_or_create(&error_labels).set(*count as i64);
            }
        }

        registry.register("consumer_processed", "Messages processed", processed);
        registry.register("consumer_last_success_at", "Last successful processing (unix timestamp)", last_success_at);
        registry.register("consumer_avg_duration_milliseconds", "Average processing duration in milliseconds", avg_duration);
        registry.register("consumer_errors", "Consumer error count by message", errors);
    }
}
