use std::time::Duration;

pub struct JwtConfig {
    pub secret: String,
    pub expiry: Duration,
}

pub struct AuthConfig {
    pub enabled: bool,
    pub tolerance: Duration,
    pub jwt: JwtConfig,
}

impl AuthConfig {
    pub fn new(enabled: bool, tolerance: Duration, jwt: JwtConfig) -> Self {
        Self { enabled, tolerance, jwt }
    }
}
