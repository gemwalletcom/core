use primitives::Chain;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    #[allow(dead_code)]
    pub chain: Chain,
    pub timeout: u64,
}
