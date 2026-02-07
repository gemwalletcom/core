use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub timeout: Duration,
    pub catchup_reload_interval: i64,
    pub min_check: Duration,
    pub max_check: Duration,
}
