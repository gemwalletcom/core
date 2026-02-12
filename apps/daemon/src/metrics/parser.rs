use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ParserLabels {
    chain: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ParserErrorLabels {
    chain: String,
    error: String,
}

#[derive(Default)]
struct ParserState {
    current_block: i64,
    latest_block: i64,
    is_enabled: bool,
    updated_at: i64,
    errors: HashMap<String, u64>,
}

pub struct ParserMetrics {
    chains: Mutex<HashMap<String, ParserState>>,
}

impl ParserMetrics {
    pub fn new() -> Self {
        Self {
            chains: Mutex::new(HashMap::new()),
        }
    }

    pub fn update_state(&self, chain: &str, current_block: i64, latest_block: i64, is_enabled: bool) {
        let mut chains = self.chains.lock().unwrap();
        let state = chains.entry(chain.to_string()).or_default();
        state.current_block = current_block;
        state.latest_block = latest_block;
        state.is_enabled = is_enabled;
        state.updated_at = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    }

    pub fn record_error(&self, chain: &str, error: &str) {
        let mut chains = self.chains.lock().unwrap();
        let state = chains.entry(chain.to_string()).or_default();
        *state.errors.entry(super::sanitize_error_message(error)).or_default() += 1;
    }
}

impl MetricsProvider for ParserMetrics {
    fn register(&self, registry: &mut Registry) {
        let latest_block = Family::<ParserLabels, Gauge>::default();
        let current_block = Family::<ParserLabels, Gauge>::default();
        let is_enabled = Family::<ParserLabels, Gauge>::default();
        let updated_at = Family::<ParserLabels, Gauge>::default();
        let errors = Family::<ParserLabels, Gauge>::default();
        let error_detail = Family::<ParserErrorLabels, Gauge>::default();

        let chains = self.chains.lock().unwrap();
        for (chain, state) in chains.iter() {
            let labels = ParserLabels { chain: chain.clone() };

            current_block.get_or_create(&labels).set(state.current_block);
            latest_block.get_or_create(&labels).set(state.latest_block);
            is_enabled.get_or_create(&labels).set(state.is_enabled as i64);
            updated_at.get_or_create(&labels).set(state.updated_at);

            let total_errors: u64 = state.errors.values().sum();
            errors.get_or_create(&labels).set(total_errors as i64);

            for (error, count) in &state.errors {
                let error_labels = ParserErrorLabels {
                    chain: chain.clone(),
                    error: error.clone(),
                };
                error_detail.get_or_create(&error_labels).set(*count as i64);
            }
        }

        registry.register("parser_state_latest_block", "Parser latest block", latest_block);
        registry.register("parser_state_current_block", "Parser current block", current_block);
        registry.register("parser_state_is_enabled", "Parser is enabled", is_enabled);
        registry.register("parser_state_updated_at", "Parser updated at", updated_at);
        registry.register("parser_errors", "Parser errors encountered", errors);
        registry.register("parser_error_detail", "Parser error count by message", error_detail);
    }
}
