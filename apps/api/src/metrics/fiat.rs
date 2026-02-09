use metrics::histogram;
use primitives::{FiatQuote, FiatQuotes};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

const FIAT_AMOUNT_BUCKETS: [f64; 6] = [50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0];

static FIAT_QUOTE_LATENCY: OnceLock<Family<FiatQuoteLabels, Histogram>> = OnceLock::new();
static FIAT_QUOTE_SUCCESS: OnceLock<Family<FiatQuoteLabels, Counter>> = OnceLock::new();
static FIAT_QUOTE_ERROR: OnceLock<Family<FiatQuoteErrorLabels, Counter>> = OnceLock::new();
static FIAT_QUOTE_AMOUNT: OnceLock<Family<FiatQuoteLabels, Histogram>> = OnceLock::new();
static FIAT_QUOTE_URL_GENERATED: OnceLock<Family<FiatQuoteLabels, Counter>> = OnceLock::new();
static FIAT_QUOTE_ERRORS: OnceLock<Family<FiatQuoteErrorLabels, Gauge>> = OnceLock::new();
static FIAT_QUOTE_ERROR_DETAIL: OnceLock<Family<FiatQuoteErrorDetailLabels, Gauge>> = OnceLock::new();
static FIAT_QUOTE_LAST_ERROR_AT: OnceLock<Family<FiatQuoteErrorLabels, Gauge>> = OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct FiatQuoteLabels {
    provider: String,
    asset_id: String,
    quote_type: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct FiatQuoteErrorLabels {
    provider: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct FiatQuoteErrorDetailLabels {
    provider: String,
    error: String,
}

pub fn init_fiat_metrics(registry: &mut Registry) {
    let latency = Family::<FiatQuoteLabels, Histogram>::new_with_constructor(histogram::latency);
    let success = Family::<FiatQuoteLabels, Counter>::default();
    let error = Family::<FiatQuoteErrorLabels, Counter>::default();
    let amount = Family::<FiatQuoteLabels, Histogram>::new_with_constructor(|| Histogram::new(FIAT_AMOUNT_BUCKETS));
    let url_generated = Family::<FiatQuoteLabels, Counter>::default();

    let errors = Family::<FiatQuoteErrorLabels, Gauge>::default();
    let error_detail = Family::<FiatQuoteErrorDetailLabels, Gauge>::default();
    let last_error_at = Family::<FiatQuoteErrorLabels, Gauge>::default();

    registry.register("fiat_quote_latency", "Fiat provider quote latency in seconds", latency.clone());
    registry.register("fiat_quote_success", "Successful fiat quotes", success.clone());
    registry.register("fiat_quote_error", "Failed fiat quotes", error.clone());
    registry.register("fiat_quote_amount", "Fiat quote amount distribution", amount.clone());
    registry.register("fiat_quote_url_generated", "Fiat quote URLs generated", url_generated.clone());
    registry.register("fiat_quote_errors", "Total fiat quote errors", errors.clone());
    registry.register("fiat_quote_error_detail", "Fiat quote error details by provider and message", error_detail.clone());
    registry.register("fiat_quote_last_error_at", "Last fiat quote error timestamp (unix)", last_error_at.clone());

    FIAT_QUOTE_LATENCY.set(latency).ok();
    FIAT_QUOTE_SUCCESS.set(success).ok();
    FIAT_QUOTE_ERROR.set(error).ok();
    FIAT_QUOTE_AMOUNT.set(amount).ok();
    FIAT_QUOTE_URL_GENERATED.set(url_generated).ok();
    FIAT_QUOTE_ERRORS.set(errors).ok();
    FIAT_QUOTE_ERROR_DETAIL.set(error_detail).ok();
    FIAT_QUOTE_LAST_ERROR_AT.set(last_error_at).ok();
}

pub fn metrics_fiat_quotes(quotes: &FiatQuotes) {
    for quote in &quotes.quotes {
        let labels = FiatQuoteLabels {
            provider: quote.provider.id.clone(),
            asset_id: quote.asset_id.clone(),
            quote_type: quote.quote_type.as_ref().to_string(),
        };

        if let Some(latency) = FIAT_QUOTE_LATENCY.get() {
            latency.get_or_create(&labels).observe(quote.latency as f64 / 1000.0);
        }
        if let Some(success) = FIAT_QUOTE_SUCCESS.get() {
            success.get_or_create(&labels).inc();
        }
        if let Some(amount) = FIAT_QUOTE_AMOUNT.get() {
            amount.get_or_create(&labels).observe(quote.fiat_amount);
        }
    }

    for error in &quotes.errors {
        if let Some(provider) = &error.provider {
            let labels = FiatQuoteErrorLabels { provider: provider.clone() };

            if let Some(error_metric) = FIAT_QUOTE_ERROR.get() {
                error_metric.get_or_create(&labels).inc();
            }
            if let Some(errors) = FIAT_QUOTE_ERRORS.get() {
                errors.get_or_create(&labels).inc();
            }

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if let Some(last_error_at) = FIAT_QUOTE_LAST_ERROR_AT.get() {
                last_error_at.get_or_create(&labels).set(now);
            }

            let detail_labels = FiatQuoteErrorDetailLabels {
                provider: provider.clone(),
                error: error.error.clone(),
            };
            if let Some(error_detail) = FIAT_QUOTE_ERROR_DETAIL.get() {
                error_detail.get_or_create(&detail_labels).set(now);
            }
        }
    }
}

pub fn metrics_fiat_quote_url(quote: &FiatQuote) {
    let labels = FiatQuoteLabels {
        provider: quote.provider.id.clone(),
        asset_id: quote.asset_id.clone(),
        quote_type: quote.quote_type.as_ref().to_string(),
    };

    if let Some(url_generated) = FIAT_QUOTE_URL_GENERATED.get() {
        url_generated.get_or_create(&labels).inc();
    }
}
