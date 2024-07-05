use crate::block_explorer::{BlockExplorer, Metadata};

pub struct Viewblock {
    pub meta: Metadata,
}

impl Viewblock {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Viewblock",
                base_url: "https://viewblock.io/thorchain",
            },
        })
    }
}

impl BlockExplorer for Viewblock {
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
        None
    }
}

pub struct RuneScan {
    pub meta: Metadata,
}

impl RuneScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "RuneScan",
                base_url: "https://runescan.io",
            },
        })
    }
}
impl BlockExplorer for RuneScan {
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
        None
    }
}
