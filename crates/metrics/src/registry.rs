use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;

#[derive(Debug)]
pub struct MetricsRegistry {
    registry: Registry,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self { registry: Registry::default() }
    }

    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            registry: Registry::with_prefix(prefix),
        }
    }

    pub fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }

    pub fn encode(&self) -> String {
        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        buffer
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}
