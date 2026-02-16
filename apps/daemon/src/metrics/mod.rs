pub mod consumer;
pub mod job;
pub mod parser;
pub mod price;

use std::sync::Arc;
use std::time::SystemTime;

use metrics::MetricsRegistry;
use prometheus_client::registry::Registry;
use rocket::response::content::RawText;
use rocket::{State, get};

const MAX_ERROR_LENGTH: usize = 200;

pub fn sanitize_error_message(error: &str) -> String {
    let truncated = match error.char_indices().nth(MAX_ERROR_LENGTH) {
        Some((idx, _)) => &error[..idx],
        None => error,
    };
    truncated.replace('\n', " ").replace('\r', "")
}

pub fn now_unix() -> u64 {
    SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

pub trait MetricsProvider: Send + Sync {
    fn register(&self, registry: &mut Registry);
}

pub struct Metrics {
    providers: Vec<Arc<dyn MetricsProvider>>,
}

impl Metrics {
    pub fn new(providers: Vec<Arc<dyn MetricsProvider>>) -> Self {
        Self { providers }
    }
}

impl MetricsProvider for Metrics {
    fn register(&self, registry: &mut Registry) {
        for provider in &self.providers {
            provider.register(registry);
        }
    }
}

#[get("/")]
pub fn get_metrics(provider: &State<Arc<dyn MetricsProvider>>) -> RawText<String> {
    let mut registry = MetricsRegistry::new();
    provider.register(registry.registry_mut());
    RawText(registry.encode())
}
