use crate::block_explorer::{BlockExplorer, Metadata};

pub struct TronScan {
    pub meta: Metadata,
}

impl TronScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "TRONSCAN",
                base_url: "https://tronscan.org",
            },
        })
    }
}

impl BlockExplorer for TronScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/#/transaction/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/#/address/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        Some(format!("{}/#/token20/{}", self.meta.base_url, _token))
    }
    fn get_validator_url(&self, validator: &str) -> Option<String> {
        self.get_address_url(validator).into()
    }
}
