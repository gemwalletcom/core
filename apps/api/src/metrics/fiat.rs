use metrics::histogram;
use primitives::{FiatQuote, FiatQuotes};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::time::{SystemTime, UNIX_EPOCH};

const FIAT_AMOUNT_BUCKETS: [f64; 6] = [50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0];

pub struct FiatMetrics {
    latency: Family<FiatQuoteLabels, Histogram>,
    success: Family<FiatQuoteLabels, Counter>,
    error: Family<FiatQuoteErrorLabels, Counter>,
    amount: Family<FiatQuoteLabels, Histogram>,
    url_generated: Family<FiatQuoteLabels, Counter>,
    errors: Family<FiatQuoteErrorLabels, Gauge>,
    error_detail: Family<FiatQuoteErrorDetailLabels, Gauge>,
    last_error_at: Family<FiatQuoteErrorLabels, Gauge>,
}

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

impl FiatMetrics {
    pub fn new() -> Self {
        Self {
            latency: Family::new_with_constructor(histogram::latency),
            success: Family::default(),
            error: Family::default(),
            amount: Family::new_with_constructor(|| Histogram::new(FIAT_AMOUNT_BUCKETS)),
            url_generated: Family::default(),
            errors: Family::default(),
            error_detail: Family::default(),
            last_error_at: Family::default(),
        }
    }

    pub fn register(&self, registry: &mut Registry) {
        registry.register("fiat_quote_latency", "Fiat provider quote latency in seconds", self.latency.clone());
        registry.register("fiat_quote_success", "Successful fiat quotes", self.success.clone());
        registry.register("fiat_quote_error", "Failed fiat quotes", self.error.clone());
        registry.register("fiat_quote_amount", "Fiat quote amount distribution", self.amount.clone());
        registry.register("fiat_quote_url_generated", "Fiat quote URLs generated", self.url_generated.clone());
        registry.register("fiat_quote_errors", "Total fiat quote errors", self.errors.clone());
        registry.register("fiat_quote_error_detail", "Fiat quote error details by provider and message", self.error_detail.clone());
        registry.register("fiat_quote_last_error_at", "Last fiat quote error timestamp (unix)", self.last_error_at.clone());
    }

    pub fn record_quotes(&self, quotes: &FiatQuotes) {
        for quote in &quotes.quotes {
            let labels = FiatQuoteLabels {
                provider: quote.provider.id.clone(),
                asset_id: quote.asset_id.clone(),
                quote_type: quote.quote_type.as_ref().to_string(),
            };

            self.latency.get_or_create(&labels).observe(quote.latency as f64 / 1000.0);
            self.success.get_or_create(&labels).inc();
            self.amount.get_or_create(&labels).observe(quote.fiat_amount);
        }

        for error in &quotes.errors {
            if let Some(provider) = &error.provider {
                let labels = FiatQuoteErrorLabels { provider: provider.clone() };

                self.error.get_or_create(&labels).inc();
                self.errors.get_or_create(&labels).inc();

                let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
                self.last_error_at.get_or_create(&labels).set(now);

                let detail_labels = FiatQuoteErrorDetailLabels {
                    provider: provider.clone(),
                    error: error.error.clone(),
                };
                self.error_detail.get_or_create(&detail_labels).set(now);
            }
        }
    }

    pub fn record_quote_url(&self, quote: &FiatQuote) {
        let labels = FiatQuoteLabels {
            provider: quote.provider.id.clone(),
            asset_id: quote.asset_id.clone(),
            quote_type: quote.quote_type.as_ref().to_string(),
        };

        self.url_generated.get_or_create(&labels).inc();
    }
}
