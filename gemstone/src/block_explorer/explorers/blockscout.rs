use crate::block_explorer::{BlockExplorer, Metadata};
use primitives::Chain;
pub struct BlockScout {
    pub meta: Metadata,
}

impl BlockScout {
    pub fn new(chain: Chain) -> Self {
        match chain {
            Chain::Celo => Self {
                meta: Metadata {
                    name: "BlockScout",
                    base_url: "https://explorer.celo.org/mainnet",
                },
            },
            _ => todo!(),
        }
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
