use crate::block_explorer::{BlockExplorer, Metadata};

pub struct ChainflipScan {
    pub meta: Metadata,
}

impl ChainflipScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Chainflip",
                base_url: "https://scan.chainflip.io",
            },
        })
    }
}

impl BlockExplorer for ChainflipScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        // it's not hash but swap id
        format!("{}/swaps/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, _address: &str) -> String {
        "".to_string()
    }
}
