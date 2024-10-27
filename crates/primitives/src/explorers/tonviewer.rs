use crate::block_explorer::{BlockExplorer, Metadata};

pub struct TonViewer {
    pub meta: Metadata,
}

impl TonViewer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "TonViewer",
                base_url: "https://tonviewer.com",
            },
        })
    }
}

impl BlockExplorer for TonViewer {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/transaction/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        Some(self.get_address_url(_token))
    }
    fn get_validator_url(&self, validator: &str) -> Option<String> {
        self.get_address_url(validator).into()
    }
}
