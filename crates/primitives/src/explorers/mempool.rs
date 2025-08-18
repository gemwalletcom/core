use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub fn new() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "Mempool",
        base_url: "https://mempool.space",
        tx_path: "tx",
        address_path: "address",
        token_path: None,
        validator_path: None,
    })
}