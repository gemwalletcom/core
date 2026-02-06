use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rocket::{State, get, http::Status, routes};

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

pub async fn run_server(state: Arc<HealthState>) {
    let _ = rocket::build().manage(state).mount("/", routes![health]).launch().await;
}
