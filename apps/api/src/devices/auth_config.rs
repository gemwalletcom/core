use std::time::Duration;

pub struct JwtConfig {
    pub secret: String,
    pub expiry: Duration,
}

pub struct AuthConfig {
    pub tolerance: Duration,
    pub jwt: JwtConfig,
}

impl AuthConfig {
    pub fn new(tolerance: Duration, jwt: JwtConfig) -> Self {
        Self { tolerance, jwt }
    }
}
