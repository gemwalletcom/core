use crate::block_explorer::{BlockExplorer, Metadata};

pub struct SubScan {
    pub meta: Metadata,
}

macro_rules! subscan_url {
    ($chain:expr) => {
        concat!("https://", $chain, ".subscan.io")
    };
}

impl SubScan {
    pub fn new_polkadot() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "ScanScan",
                base_url: subscan_url!("polkadot"),
            },
        })
    }
}

impl BlockExplorer for SubScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/extrinsic/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
    fn get_validator_url(&self, _validator: &str) -> Option<String> {
        None
    }
}
