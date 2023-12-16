use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use storage::DatabaseClient;

pub struct MetricsClient {
    registry: Registry,
    parser_latest_block: Family<ParserStateLabels, Gauge>,
    parser_current_block: Family<ParserStateLabels, Gauge>,
    parser_is_enabled: Family<ParserStateLabels, Gauge>,
    parser_updated_at: Family<ParserStateLabels, Gauge>,
    database: DatabaseClient,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ParserStateLabels {
    chain: String,
}

impl MetricsClient {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);

        let parser_latest_block = Family::<ParserStateLabels, Gauge>::default();
        let parser_current_block = Family::<ParserStateLabels, Gauge>::default();
        let parser_is_enabled = Family::<ParserStateLabels, Gauge>::default();
        let parser_updated_at = Family::<ParserStateLabels, Gauge>::default();

        let mut registry = <Registry>::with_prefix("parser");
        registry.register(
            "state_latest_block",
            "Parser latest block",
            parser_latest_block.clone(),
        );
        registry.register(
            "state_current_block",
            "Parser current block",
            parser_current_block.clone(),
        );
        registry.register(
            "state_is_enabled",
            "Parser is enabled",
            parser_is_enabled.clone(),
        );
        registry.register(
            "state_updated_at",
            "Parser updated at",
            parser_updated_at.clone(),
        );

        Self {
            registry,
            parser_latest_block,
            parser_current_block,
            parser_is_enabled,
            parser_updated_at,
            database,
        }
    }
    pub fn get(&mut self) -> String {
        self.update_parser_states();

        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        buffer
    }

    pub fn update_parser_states(&mut self) {
        for state in self.database.get_parser_states().unwrap_or_default() {
            self.parser_current_block
                .get_or_create(&ParserStateLabels {
                    chain: state.clone().chain,
                })
                .set(state.current_block as i64);
            self.parser_latest_block
                .get_or_create(&ParserStateLabels {
                    chain: state.clone().chain,
                })
                .set(state.latest_block as i64);
            self.parser_is_enabled
                .get_or_create(&ParserStateLabels {
                    chain: state.clone().chain,
                })
                .set(state.is_enabled as i64);
            self.parser_updated_at
                .get_or_create(&ParserStateLabels {
                    chain: state.clone().chain,
                })
                .set(state.updated_at.timestamp());
        }
    }
}
