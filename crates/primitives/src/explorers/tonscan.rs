use crate::block_explorer::{BlockExplorer, Metadata};

pub struct TonScan {
    pub meta: Metadata,
}

impl TonScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Tonscan",
                base_url: "https://tonscan.org",
            },
        })
    }
}

impl BlockExplorer for TonScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        format!("{}/jetton/{}", self.meta.base_url, token).into()
    }
    fn get_validator_url(&self, validator: &str) -> Option<String> {
        self.get_address_url(validator).into()
    }
}
