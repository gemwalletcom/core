use std::sync::atomic::AtomicU64;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use storage::Database;

pub struct MetricsClient {
    registry: Registry,

    parser_latest_block: Family<ParserStateLabels, Gauge>,
    parser_current_block: Family<ParserStateLabels, Gauge>,
    parser_is_enabled: Family<ParserStateLabels, Gauge>,
    parser_updated_at: Family<ParserStateLabels, Gauge>,

    pricer_updated_at: Family<PricerStateLabels, Gauge>,
    pricer_price: Family<PricerStateLabels, Gauge<f64, AtomicU64>>,

    database: Database,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ParserStateLabels {
    chain: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct PricerStateLabels {
    asset_id: String,
}

impl MetricsClient {
    pub fn new(database: Database) -> Self {
        let parser_latest_block = Family::<ParserStateLabels, Gauge>::default();
        let parser_current_block = Family::<ParserStateLabels, Gauge>::default();
        let parser_is_enabled = Family::<ParserStateLabels, Gauge>::default();
        let parser_updated_at = Family::<ParserStateLabels, Gauge>::default();
        let pricer_updated_at = Family::<PricerStateLabels, Gauge>::default();
        let pricer_price = Family::<PricerStateLabels, Gauge<f64, AtomicU64>>::default();

        let mut registry = <Registry>::default();
        registry.register("parser_state_latest_block", "Parser latest block", parser_latest_block.clone());
        registry.register("parser_state_current_block", "Parser current block", parser_current_block.clone());
        registry.register("parser_state_is_enabled", "Parser is enabled", parser_is_enabled.clone());
        registry.register("parser_state_updated_at", "Parser updated at", parser_updated_at.clone());
        registry.register("pricer_updated_at", "Pricer updated at", pricer_updated_at.clone());
        registry.register("pricer_price", "Pricer price", pricer_price.clone());

        Self {
            registry,
            parser_latest_block,
            parser_current_block,
            parser_is_enabled,
            parser_updated_at,
            pricer_updated_at,
            pricer_price,
            database,
        }
    }
    pub fn get(&self) -> String {
        self.update_parser_states();
        self.update_pricer();

        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        buffer
    }

    pub fn update_parser_states(&self) {
        let states = self
            .database
            .client()
            .ok()
            .and_then(|mut c| c.parser_state().get_parser_states().ok())
            .unwrap_or_default();

        for state in states {
            self.parser_current_block
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.current_block);
            self.parser_latest_block
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.latest_block);
            self.parser_is_enabled
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.is_enabled as i64);
            self.parser_updated_at
                .get_or_create(&ParserStateLabels { chain: state.clone().chain })
                .set(state.updated_at.and_utc().timestamp());
        }
    }

    pub fn update_pricer(&self) {
        let prices = self
            .database
            .client()
            .ok()
            .and_then(|mut c| c.prices().get_prices().ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|x| x.market_cap_rank >= 1 && x.market_cap_rank <= 10);

        for price in prices {
            self.pricer_updated_at
                .get_or_create(&PricerStateLabels { asset_id: price.clone().id })
                .set(price.last_updated_at.and_utc().timestamp());

            self.pricer_price
                .get_or_create(&PricerStateLabels { asset_id: price.clone().id })
                .set(price.price);
        }
    }
}
