use crate::block_explorer::{BlockExplorer, Metadata};

pub struct HyperLiquid {
    pub meta: Metadata,
}

impl HyperLiquid {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Hyper Explorer",
                base_url: "https://hyperevm-explorer.vercel.app/",
            },
        })
    }
}

impl BlockExplorer for HyperLiquid {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_validator_url(&self, validator: &str) -> Option<String> {
        self.get_address_url(validator).into()
    }
}
