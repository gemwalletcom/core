use primitives::Chain;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    #[allow(dead_code)]
    pub chain: Chain,
    pub timeout: Duration,
}
