use crate::block_explorer::{BlockExplorer, Metadata};

static MINTSCAN_NAME: &str = "Mintscan";

macro_rules! mintscan_url {
    ($chain:expr) => {
        concat!("https://www.mintscan.io/", $chain)
    };
}

pub struct MintScan {
    pub meta: Metadata,
}

impl MintScan {
    pub fn new_cosmos() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: MINTSCAN_NAME,
                base_url: mintscan_url!("cosmos"),
            },
        })
    }

    pub fn new_osmosis() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: MINTSCAN_NAME,
                base_url: mintscan_url!("osmosis"),
            },
        })
    }

    pub fn new_celestia() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: MINTSCAN_NAME,
                base_url: mintscan_url!("celestia"),
            },
        })
    }

    pub fn new_injective() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: MINTSCAN_NAME,
                base_url: mintscan_url!("injective"),
            },
        })
    }

    pub fn new_sei() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: MINTSCAN_NAME,
                base_url: mintscan_url!("sei"),
            },
        })
    }

    pub fn new_noble() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: MINTSCAN_NAME,
                base_url: mintscan_url!("noble"),
            },
        })
    }
}

impl BlockExplorer for MintScan {
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
        format!("{}/assets/{}", self.meta.base_url, _token).into()
    }
}
