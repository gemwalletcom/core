use crate::block_explorer::{BlockExplorer, Metadata};

pub struct SolanaFM {
    pub meta: Metadata,
}

impl SolanaFM {
    pub fn new() -> Self {
        Self {
            meta: Metadata {
                name: "SolanaFM",
                base_url: "https://solana.fm",
            },
        }
    }
}

impl BlockExplorer for SolanaFM {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("{}/address/{}", self.meta.base_url, token))
    }
}
