use crate::block_explorer::{BlockExplorer, Metadata};

pub struct AptosExplorer {
    pub meta: Metadata,
}

impl AptosExplorer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Aptos",
                base_url: "https://explorer.aptoslabs.com",
            },
        })
    }
}

impl BlockExplorer for AptosExplorer {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/txn/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
}

pub struct AptosScan {
    pub meta: Metadata,
}

impl AptosScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "AptosScan",
                base_url: "https://aptoscan.com",
            },
        })
    }
}
impl BlockExplorer for AptosScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/transaction/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        Some(format!("{}/coin/{}", self.meta.base_url, _token))
    }
}
