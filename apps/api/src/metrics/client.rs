use cacher::CacherClient;
use metrics::MetricsRegistry;
use storage::Database;

pub struct MetricsClient {
    registry: MetricsRegistry,
    database: Database,
    cacher: CacherClient,
}

impl MetricsClient {
    pub fn new(database: Database, cacher: CacherClient) -> Self {
        let mut registry = MetricsRegistry::new();

        super::parser::init_parser_metrics(registry.registry_mut());
        super::price::init_price_metrics(registry.registry_mut());
        super::fiat::init_fiat_metrics(registry.registry_mut());
        super::job::init_job_metrics(registry.registry_mut());
        super::consumer::init_consumer_metrics(registry.registry_mut());

        Self { registry, database, cacher }
    }

    pub async fn get(&self) -> String {
        super::parser::update_parser_metrics(&self.database, &self.cacher).await;
        super::price::update_price_metrics(&self.database);
        super::job::update_job_metrics(&self.cacher).await;
        super::consumer::update_consumer_metrics(&self.cacher).await;

        self.registry.encode()
    }
}
