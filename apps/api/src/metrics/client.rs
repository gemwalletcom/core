use cacher::CacherClient;
use metrics::MetricsRegistry;
use storage::Database;

use super::fiat::FiatMetrics;

pub struct MetricsClient {
    pub fiat: FiatMetrics,
    database: Database,
    cacher: CacherClient,
}

impl MetricsClient {
    pub fn new(database: Database, cacher: CacherClient) -> Self {
        Self {
            fiat: FiatMetrics::new(),
            database,
            cacher,
        }
    }

    pub async fn get(&self) -> String {
        let mut registry = MetricsRegistry::new();
        self.fiat.register(registry.registry_mut());
        super::price::register(registry.registry_mut(), &self.database, &self.cacher).await;
        registry.encode()
    }
}
