use crate::block_explorer::{BlockExplorer, Metadata};

pub struct MantleExplorer {
    pub meta: Metadata,
}

impl MantleExplorer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Mantle Explorer",
                base_url: "https://explorer.mantle.xyz",
            },
        })
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
    fn get_validator_url(&self, _validator: &str) -> Option<String> {  None }
}
