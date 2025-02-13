use crate::block_explorer::{BlockExplorer, Metadata};

pub struct Cardanocan {
    pub meta: Metadata,
}

impl Cardanocan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "CardanoScan",
                base_url: "https://cardanoscan.io",
            },
        })
    }
}

impl BlockExplorer for Cardanocan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/transaction/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
}
