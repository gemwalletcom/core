use crate::block_explorer::{BlockExplorer, Metadata};

static MANTLE_NAME: &str = "Mantle Explorer";
static MANTLE_BASE_URL: &str = "https://explorer.mantle.xyz";

pub struct MantleExplorer {
    pub meta: Metadata,
}

impl MantleExplorer {
    pub fn new() -> Self {
        Self {
            meta: Metadata {
                name: MANTLE_NAME,
                base_url: MANTLE_BASE_URL,
            },
        }
    }
}

impl BlockExplorer for MantleExplorer {
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
