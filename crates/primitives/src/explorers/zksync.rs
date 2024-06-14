use crate::block_explorer::{BlockExplorer, Metadata};

pub struct ZkSync {
    pub meta: Metadata,
}

impl ZkSync {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "zkSync.io",
                base_url: "https://explorer.zksync.io",
            },
        })
    }
}

impl BlockExplorer for ZkSync {
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
        Some(self.get_address_url(_token))
    }
}
