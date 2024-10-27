use crate::block_explorer::{BlockExplorer, Metadata};

pub struct NearBlocks {
    pub meta: Metadata,
}

impl NearBlocks {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Near",
                base_url: "https://nearblocks.io",
            },
        })
    }
}

impl BlockExplorer for NearBlocks {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/txns/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> { None }
    fn get_validator_url(&self, _validator: &str) -> Option<String> {  None }
}
