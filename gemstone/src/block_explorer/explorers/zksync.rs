use crate::block_explorer::{BlockExplorer, Metadata};

static ZKSYNC_NAME: &str = "zkSync.io";
static ZKSYNC_BASE_URL: &str = "https://explorer.zksync.io";

pub struct ZkSync {
    pub meta: Metadata,
}

impl ZkSync {
    pub fn new() -> Self {
        Self {
            meta: Metadata {
                name: ZKSYNC_NAME,
                base_url: ZKSYNC_BASE_URL,
            },
        }
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
