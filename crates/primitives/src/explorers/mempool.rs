use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{GenericExplorer, Metadata};

pub fn new() -> Box<dyn BlockExplorer> {
    GenericExplorer::new(Metadata {
        name: "Mempool",
        base_url: "https://mempool.space",
        tx_path: "tx",
        address_path: "address",
        token_path: None,
        validator_path: None,
    })
}