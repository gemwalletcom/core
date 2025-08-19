use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ADDRESS_PATH};

pub fn new() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "Mempool",
        base_url: "https://mempool.space",
        tx_path: TX_PATH,
        address_path: ADDRESS_PATH,
        token_path: None,
        validator_path: None,
    })
}