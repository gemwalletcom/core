pub mod consumer;
pub mod job;
pub mod parser;

use metrics::MetricsRegistry;
use prometheus_client::registry::Registry;
use rocket::response::content::RawText;
use rocket::{State, get};
use std::sync::Arc;

const MAX_ERROR_LENGTH: usize = 200;

pub fn sanitize_error_message(error: &str) -> String {
    let truncated = match error.char_indices().nth(MAX_ERROR_LENGTH) {
        Some((idx, _)) => &error[..idx],
        None => error,
    };
    truncated.replace('\n', " ").replace('\r', "")
}

pub trait MetricsProvider: Send + Sync {
    fn register(&self, registry: &mut Registry);
}

#[get("/")]
pub fn get_metrics(provider: &State<Arc<dyn MetricsProvider>>) -> RawText<String> {
    let mut registry = MetricsRegistry::new();
    provider.register(registry.registry_mut());
    RawText(registry.encode())
}
