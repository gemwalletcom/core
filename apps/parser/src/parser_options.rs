use primitives::Chain;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub chain: Chain,
    pub timeout: u64,
}

impl ParserOptions {
    pub fn minimum_transfer_amount(&self) -> u64 {
        match self.chain {
            Chain::Tron | Chain::Xrp => 5_000,
            Chain::Stellar => 50_000,
            Chain::Polkadot => 10_000_000,
            _ => 0,
        }
    }
}
