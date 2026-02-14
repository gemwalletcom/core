pub mod fiat;

use std::sync::Arc;

use crate::responders::ApiError;
use fiat::FiatMetrics;
use metrics::MetricsRegistry;
use rocket::{State, get, response::content::RawText};

#[get("/")]
pub fn get_metrics(fiat: &State<Arc<FiatMetrics>>) -> Result<RawText<String>, ApiError> {
    let mut registry = MetricsRegistry::new();
    fiat.register(registry.registry_mut());
    Ok(RawText(registry.encode()))
}
