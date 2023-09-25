use storage::DatabaseClient;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;

pub struct MetricsClient {
    registry: Registry,
    parser_state: Family::<ParserStateLabels, Counter>,
    database: DatabaseClient,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ParserStateLabels {
  chain: String,
  is_enabled: String,
  current_block: i32,
  latest_block: i32,
}

impl MetricsClient {
    pub async fn new(
        database_url: &str,
    ) -> Self {
        let database = DatabaseClient::new(database_url);

        let parser_state = Family::<ParserStateLabels, Counter>::default();

        let mut registry = <Registry>::with_prefix("parser");
        registry.register("state", "Parser state", parser_state.clone());

        Self {
            registry,
            parser_state,
            database,
        }
    }
    pub fn get(&mut self) -> String {
        self.update_parser_states();

        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        return buffer
    }

    pub fn update_parser_states(&mut self) {
        for state in self.database.get_parser_states().unwrap_or_default() {
            self.parser_state.get_or_create(
                &ParserStateLabels { chain: state.chain, is_enabled: state.is_enabled.to_string(), latest_block: state.latest_block, current_block: state.current_block },
            ).get();
        }
    }
}