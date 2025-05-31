use primitives::Chain;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub chain: Chain,
    pub timeout: u64,
}

impl ParserOptions {}
