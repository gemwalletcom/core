use std::collections::HashMap;
use std::sync::Mutex;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct JobLabels {
    service: String,
    job_name: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct JobErrorLabels {
    service: String,
    job_name: String,
    error: String,
}

#[derive(Default)]
struct JobState {
    interval: u64,
    duration: u64,
    last_success: Option<u64>,
    errors: HashMap<String, u64>,
}

pub struct JobMetrics {
    service: String,
    jobs: Mutex<HashMap<String, JobState>>,
}

impl JobMetrics {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn report(&self, name: &str, interval: u64, duration: u64, success: bool, error: Option<String>) {
        let mut jobs = self.jobs.lock().unwrap();
        let state = jobs.entry(name.to_string()).or_default();
        let timestamp = super::now_unix();

        state.interval = interval;
        state.duration = duration;

        if success {
            state.last_success = Some(timestamp);
        } else if let Some(error) = error {
            *state.errors.entry(super::sanitize_error_message(&error)).or_default() += 1;
        }
    }
}

impl MetricsProvider for JobMetrics {
    fn register(&self, registry: &mut Registry) {
        let last_success_at = Family::<JobLabels, Gauge>::default();
        let interval = Family::<JobLabels, Gauge>::default();
        let duration = Family::<JobLabels, Gauge>::default();
        let errors = Family::<JobErrorLabels, Gauge>::default();

        let jobs = self.jobs.lock().unwrap();
        for (name, state) in jobs.iter() {
            let labels = JobLabels {
                service: self.service.clone(),
                job_name: name.clone(),
            };

            interval.get_or_create(&labels).set(state.interval as i64);
            duration.get_or_create(&labels).set(state.duration as i64);

            if let Some(ts) = state.last_success {
                last_success_at.get_or_create(&labels).set(ts as i64);
            }

            for (error, count) in &state.errors {
                let error_labels = JobErrorLabels {
                    service: self.service.clone(),
                    job_name: name.clone(),
                    error: error.clone(),
                };
                errors.get_or_create(&error_labels).set(*count as i64);
            }
        }

        registry.register("job_last_success_at", "Last successful job run (unix timestamp)", last_success_at);
        registry.register("job_interval_seconds", "Job interval in seconds", interval);
        registry.register("job_duration_milliseconds", "Last job duration in milliseconds", duration);
        registry.register("job_errors", "Job error count by message", errors);
    }
}
