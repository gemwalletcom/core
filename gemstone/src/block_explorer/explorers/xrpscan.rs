use crate::block_explorer::{BlockExplorer, Metadata};

pub struct XrpScan {
    pub meta: Metadata,
}

impl XrpScan {
    pub fn new() -> Self {
        Self {
            meta: Metadata {
                name: "XrpScan",
                base_url: "https://xrpscan.com",
            },
        }
    }
}

impl BlockExplorer for XrpScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
}
