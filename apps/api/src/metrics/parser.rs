use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::OnceLock;
use storage::Database;

static PARSER_LATEST_BLOCK: OnceLock<Family<ParserStateLabels, Gauge>> = OnceLock::new();
static PARSER_CURRENT_BLOCK: OnceLock<Family<ParserStateLabels, Gauge>> = OnceLock::new();
static PARSER_IS_ENABLED: OnceLock<Family<ParserStateLabels, Gauge>> = OnceLock::new();
static PARSER_UPDATED_AT: OnceLock<Family<ParserStateLabels, Gauge>> = OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ParserStateLabels {
    chain: String,
}

pub fn init_parser_metrics(registry: &mut Registry) {
    let latest_block = Family::<ParserStateLabels, Gauge>::default();
    let current_block = Family::<ParserStateLabels, Gauge>::default();
    let is_enabled = Family::<ParserStateLabels, Gauge>::default();
    let updated_at = Family::<ParserStateLabels, Gauge>::default();

    registry.register("parser_state_latest_block", "Parser latest block", latest_block.clone());
    registry.register("parser_state_current_block", "Parser current block", current_block.clone());
    registry.register("parser_state_is_enabled", "Parser is enabled", is_enabled.clone());
    registry.register("parser_state_updated_at", "Parser updated at", updated_at.clone());

    PARSER_LATEST_BLOCK.set(latest_block).ok();
    PARSER_CURRENT_BLOCK.set(current_block).ok();
    PARSER_IS_ENABLED.set(is_enabled).ok();
    PARSER_UPDATED_AT.set(updated_at).ok();
}

pub fn update_parser_metrics(database: &Database) {
    let states = database
        .client()
        .ok()
        .and_then(|mut c| c.parser_state().get_parser_states().ok())
        .unwrap_or_default();

    for state in states {
        if let Some(current_block) = PARSER_CURRENT_BLOCK.get() {
            current_block
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.current_block);
        }
        if let Some(latest_block) = PARSER_LATEST_BLOCK.get() {
            latest_block
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.latest_block);
        }
        if let Some(is_enabled) = PARSER_IS_ENABLED.get() {
            is_enabled
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.is_enabled as i64);
        }
        if let Some(updated_at) = PARSER_UPDATED_AT.get() {
            updated_at
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.updated_at.and_utc().timestamp());
        }
    }
}
