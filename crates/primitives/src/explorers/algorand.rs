use crate::block_explorer::{BlockExplorer, Metadata};

pub struct AlgorandAllo {
    pub meta: Metadata,
}

impl AlgorandAllo {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Allo",
                base_url: "https://allo.info",
            },
        })
    }
}

impl BlockExplorer for AlgorandAllo {
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

    fn get_validator_url(&self, validator: &str) -> Option<String> {
        self.get_address_url(validator).into()
    }
}
