use metrics::MetricsRegistry;
use storage::Database;

pub struct MetricsClient {
    registry: MetricsRegistry,
    database: Database,
}

impl MetricsClient {
    pub fn new(database: Database) -> Self {
        let mut registry = MetricsRegistry::new();

        super::parser::init_parser_metrics(registry.registry_mut());
        super::price::init_price_metrics(registry.registry_mut());
        super::fiat::init_fiat_metrics(registry.registry_mut());

        Self { registry, database }
    }

    pub fn get(&self) -> String {
        super::parser::update_parser_metrics(&self.database);
        super::price::update_price_metrics(&self.database);

        self.registry.encode()
    }
}
