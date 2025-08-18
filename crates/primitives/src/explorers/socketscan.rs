use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ADDRESS_PATH};

pub struct SocketScan;

impl SocketScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "SocketScan",
            base_url: "https://socketscan.io",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}
