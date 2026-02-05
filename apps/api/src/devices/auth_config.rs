use std::time::Duration;

pub struct AuthConfig {
    pub enabled: bool,
    pub tolerance: Duration,
}

impl AuthConfig {
    pub fn new(enabled: bool, tolerance: Duration) -> Self {
        Self { enabled, tolerance }
    }
}
