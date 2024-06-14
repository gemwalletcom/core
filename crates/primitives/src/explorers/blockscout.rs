use crate::block_explorer::{BlockExplorer, Metadata};
pub struct BlockScout {
    pub meta: Metadata,
}

impl BlockScout {
    pub fn new_celo() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "BlockScout",
                base_url: "https://explorer.celo.org/mainnet",
            },
        })
    }
}

impl BlockExplorer for BlockScout {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        Some(format!("{}/token/{}", self.meta.base_url, _token))
    }
}
