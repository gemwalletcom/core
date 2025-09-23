use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, Explorer, Metadata};

pub struct SubScan;

macro_rules! subscan_url {
    ($chain:expr) => {
        concat!("https://", $chain, ".subscan.io")
    };
}

impl SubScan {
    pub fn new_polkadot() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "SubScan",
            base_url: subscan_url!("polkadot"),
            tx_path: "/extrinsic",
            address_path: ACCOUNT_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}
