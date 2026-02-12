use metrics::MetricsRegistry;

use super::fiat::FiatMetrics;

pub struct MetricsClient {
    pub fiat: FiatMetrics,
}

impl MetricsClient {
    pub fn new() -> Self {
        Self { fiat: FiatMetrics::new() }
    }

    pub async fn get(&self) -> String {
        let mut registry = MetricsRegistry::new();
        self.fiat.register(registry.registry_mut());
        registry.encode()
    }
}
