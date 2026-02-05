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
static CONSUMER_ERROR_DETAIL: OnceLock<Family<ConsumerErrorLabels, Gauge>> = OnceLock::new();
static CONSUMER_LAST_ERROR_AT: OnceLock<Family<ConsumerErrorLabels, Gauge>> = OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ConsumerLabels {
    consumer: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ConsumerErrorLabels {
    consumer: String,
    error: String,
}

pub fn init_consumer_metrics(registry: &mut Registry) {
    let processed = Family::<ConsumerLabels, Gauge>::default();
    let errors = Family::<ConsumerLabels, Gauge>::default();
    let last_success = Family::<ConsumerLabels, Gauge>::default();
    let avg_duration = Family::<ConsumerLabels, Gauge>::default();
    let error_detail = Family::<ConsumerErrorLabels, Gauge>::default();
    let error_at = Family::<ConsumerErrorLabels, Gauge>::default();

    registry.register("consumer_processed", "Messages processed", processed.clone());
    registry.register("consumer_errors", "Errors encountered", errors.clone());
    registry.register("consumer_last_success_at", "Last successful processing (unix timestamp)", last_success.clone());
    registry.register("consumer_avg_duration_ms", "Average processing duration in milliseconds", avg_duration.clone());
    registry.register("consumer_error_detail", "Error occurrence count per consumer and error message", error_detail.clone());
    registry.register("consumer_last_error_at", "Last error timestamp (unix)", error_at.clone());

    CONSUMER_PROCESSED.set(processed).ok();
    CONSUMER_ERRORS.set(errors).ok();
    CONSUMER_LAST_SUCCESS_AT.set(last_success).ok();
    CONSUMER_AVG_DURATION_MS.set(avg_duration).ok();
    CONSUMER_ERROR_DETAIL.set(error_detail).ok();
    CONSUMER_LAST_ERROR_AT.set(error_at).ok();
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
        for err in &status.errors {
            let error_labels = ConsumerErrorLabels {
                consumer: name.to_string(),
                error: err.message.clone(),
            };
            if let Some(family) = CONSUMER_ERROR_DETAIL.get() {
                family.get_or_create(&error_labels).set(err.count as i64);
            }
            if let Some(family) = CONSUMER_LAST_ERROR_AT.get() {
                family.get_or_create(&error_labels).set(err.timestamp as i64);
            }
        }
    }
}
