use cacher::CacherClient;
use primitives::JobStatus;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::OnceLock;

const JOBS_STATUS_PREFIX: &str = "jobs:status:";

static JOB_LAST_SUCCESS_AT: OnceLock<Family<JobLabels, Gauge>> = OnceLock::new();
static JOB_INTERVAL_SECONDS: OnceLock<Family<JobLabels, Gauge>> = OnceLock::new();
static JOB_LAST_ERROR_AT: OnceLock<Family<JobLabels, Gauge>> = OnceLock::new();
static JOB_DURATION_MS: OnceLock<Family<JobLabels, Gauge>> = OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct JobLabels {
    service: String,
    job_name: String,
}

pub fn init_job_metrics(registry: &mut Registry) {
    let last_success = Family::<JobLabels, Gauge>::default();
    let interval = Family::<JobLabels, Gauge>::default();
    let last_error = Family::<JobLabels, Gauge>::default();
    let duration = Family::<JobLabels, Gauge>::default();

    registry.register("job_last_success_at", "Last successful job run (unix timestamp)", last_success.clone());
    registry.register("job_interval_seconds", "Job interval in seconds", interval.clone());
    registry.register("job_last_error_at", "Last job error (unix timestamp)", last_error.clone());
    registry.register("job_duration_ms", "Last job duration in milliseconds", duration.clone());

    JOB_LAST_SUCCESS_AT.set(last_success).ok();
    JOB_INTERVAL_SECONDS.set(interval).ok();
    JOB_LAST_ERROR_AT.set(last_error).ok();
    JOB_DURATION_MS.set(duration).ok();
}

pub async fn update_job_metrics(cacher: &CacherClient) {
    let keys = match cacher.keys(&format!("{}*", JOBS_STATUS_PREFIX)).await {
        Ok(k) => k,
        Err(_) => return,
    };

    for key in &keys {
        let name = key.strip_prefix(JOBS_STATUS_PREFIX).unwrap_or(key);
        let Ok(status) = cacher.get_value::<JobStatus>(key).await else {
            continue;
        };
        let (service, job_name) = name.split_once(':').unwrap_or(("unknown", name));
        let labels = JobLabels { service: service.to_string(), job_name: job_name.to_string() };

        if let Some(family) = JOB_INTERVAL_SECONDS.get() {
            family.get_or_create(&labels).set(status.interval as i64);
        }
        if let Some(family) = JOB_DURATION_MS.get() {
            family.get_or_create(&labels).set(status.duration as i64);
        }
        if let (Some(family), Some(ts)) = (JOB_LAST_SUCCESS_AT.get(), status.last_success) {
            family.get_or_create(&labels).set(ts as i64);
        }
        if let (Some(family), Some(ts)) = (JOB_LAST_ERROR_AT.get(), status.last_error_at) {
            family.get_or_create(&labels).set(ts as i64);
        }
    }
}
