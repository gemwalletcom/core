use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rocket::{State, get, http::Status, routes};

use crate::metrics::{self, MetricsProvider};

pub struct HealthState {
    ready: AtomicBool,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthState {
    pub fn new() -> Self {
        Self { ready: AtomicBool::new(false) }
    }

    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::Relaxed);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}

#[get("/health")]
fn health(state: &State<Arc<HealthState>>) -> Status {
    if state.is_ready() { Status::Ok } else { Status::ServiceUnavailable }
}

pub async fn run_server(state: Arc<HealthState>, metrics_provider: Arc<dyn MetricsProvider>) {
    let _ = rocket::build()
        .manage(state)
        .manage(metrics_provider)
        .mount("/", routes![health])
        .mount("/metrics", routes![metrics::get_metrics])
        .launch()
        .await;
}

pub fn spawn_server(metrics_provider: Arc<dyn MetricsProvider>) -> Arc<HealthState> {
    let state = Arc::new(HealthState::new());
    tokio::spawn({
        let state = state.clone();
        async move { run_server(state, metrics_provider).await }
    });
    state
}
