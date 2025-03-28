use crate::block_explorer::{BlockExplorer, Metadata};

pub struct Mempool {
    pub meta: Metadata,
}

impl Mempool {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Mempool",
                base_url: "https://mempool.space",
            },
        })
    }
}

impl BlockExplorer for Mempool {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
}
