use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, Explorer, Metadata};

pub struct SubScan;

impl SubScan {
    pub fn new_polkadot() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "SubScan",
            base_url: "https://assethub-polkadot.subscan.io",
            tx_path: "/extrinsic",
            address_path: ACCOUNT_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}
