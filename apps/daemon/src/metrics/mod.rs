pub mod consumer;
pub mod job;
pub mod parser;

use metrics::MetricsRegistry;
use prometheus_client::registry::Registry;
use rocket::response::content::RawText;
use rocket::{State, get};
use std::sync::Arc;

pub trait MetricsProvider: Send + Sync {
    fn register(&self, registry: &mut Registry);
}

#[get("/")]
pub fn get_metrics(provider: &State<Arc<dyn MetricsProvider>>) -> RawText<String> {
    let mut registry = MetricsRegistry::new();
    provider.register(registry.registry_mut());
    RawText(registry.encode())
}
