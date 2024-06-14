use crate::block_explorer::{BlockExplorer, Metadata};

pub struct SuiScan {
    pub meta: Metadata,
}

impl SuiScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "SuiScan",
                base_url: "https://suiscan.xyz/mainnet",
            },
        })
    }
}

impl BlockExplorer for SuiScan {
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
        Some(format!("{}/account/{}", self.meta.base_url, _token))
    }
}

pub struct SuiVision {
    pub meta: Metadata,
}

impl SuiVision {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "SuiVision",
                base_url: "https://suivision.xyz",
            },
        })
    }
}
impl BlockExplorer for SuiVision {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/txblock/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        Some(format!("{}/coin/{}", self.meta.base_url, _token))
    }
}
