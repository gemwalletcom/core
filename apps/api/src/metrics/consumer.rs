use cacher::CacherClient;
use primitives::ConsumerStatus;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::OnceLock;

const CONSUMERS_STATUS_PREFIX: &str = "consumers:status:";

static CONSUMER_PROCESSED: OnceLock<Family<ConsumerLabels, Gauge>> = OnceLock::new();
static CONSUMER_ERRORS: OnceLock<Family<ConsumerLabels, Gauge>> = OnceLock::new();
static CONSUMER_LAST_SUCCESS_AT: OnceLock<Family<ConsumerLabels, Gauge>> = OnceLock::new();
static CONSUMER_AVG_DURATION_MS: OnceLock<Family<ConsumerLabels, Gauge>> = OnceLock::new();
static CONSUMER_UNIQUE_ERRORS: OnceLock<Family<ConsumerLabels, Gauge>> = OnceLock::new();
static CONSUMER_LAST_ERROR_AT: OnceLock<Family<ConsumerLabels, Gauge>> = OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ConsumerLabels {
    consumer: String,
}

pub fn init_consumer_metrics(registry: &mut Registry) {
    let processed = Family::<ConsumerLabels, Gauge>::default();
    let errors = Family::<ConsumerLabels, Gauge>::default();
    let last_success = Family::<ConsumerLabels, Gauge>::default();
    let avg_duration = Family::<ConsumerLabels, Gauge>::default();
    let unique_errors = Family::<ConsumerLabels, Gauge>::default();
    let last_error = Family::<ConsumerLabels, Gauge>::default();

    registry.register("consumer_processed", "Messages processed", processed.clone());
    registry.register("consumer_errors", "Errors encountered", errors.clone());
    registry.register("consumer_last_success_at", "Last successful processing (unix timestamp)", last_success.clone());
    registry.register("consumer_avg_duration_ms", "Average processing duration in milliseconds", avg_duration.clone());
    registry.register("consumer_unique_errors", "Number of unique error types", unique_errors.clone());
    registry.register("consumer_last_error_at", "Most recent error (unix timestamp)", last_error.clone());

    CONSUMER_PROCESSED.set(processed).ok();
    CONSUMER_ERRORS.set(errors).ok();
    CONSUMER_LAST_SUCCESS_AT.set(last_success).ok();
    CONSUMER_AVG_DURATION_MS.set(avg_duration).ok();
    CONSUMER_UNIQUE_ERRORS.set(unique_errors).ok();
    CONSUMER_LAST_ERROR_AT.set(last_error).ok();
}

pub async fn update_consumer_metrics(cacher: &CacherClient) {
    let keys = match cacher.keys(&format!("{}*", CONSUMERS_STATUS_PREFIX)).await {
        Ok(k) => k,
        Err(_) => return,
    };

    for key in &keys {
        let name = key.strip_prefix(CONSUMERS_STATUS_PREFIX).unwrap_or(key);
        let Ok(status) = cacher.get_value::<ConsumerStatus>(key).await else {
            continue;
        };
        let labels = ConsumerLabels { consumer: name.to_string() };

        if let Some(family) = CONSUMER_PROCESSED.get() {
            family.get_or_create(&labels).set(status.total_processed as i64);
        }
        if let Some(family) = CONSUMER_ERRORS.get() {
            family.get_or_create(&labels).set(status.total_errors as i64);
        }
        if let (Some(family), Some(ts)) = (CONSUMER_LAST_SUCCESS_AT.get(), status.last_success) {
            family.get_or_create(&labels).set(ts as i64);
        }
        if let Some(family) = CONSUMER_AVG_DURATION_MS.get() {
            family.get_or_create(&labels).set(status.avg_duration as i64);
        }
        if let Some(family) = CONSUMER_UNIQUE_ERRORS.get() {
            family.get_or_create(&labels).set(status.errors.len() as i64);
        }
        if let Some(family) = CONSUMER_LAST_ERROR_AT.get() {
            let last_error_at = status.errors.iter().map(|e| e.last_seen).max();
            if let Some(ts) = last_error_at {
                family.get_or_create(&labels).set(ts as i64);
            }
        }
    }
}
