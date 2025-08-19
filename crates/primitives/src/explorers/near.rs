use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, ADDRESS_PATH, TXNS_PATH};

pub struct NearBlocks;

impl NearBlocks {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Near",
            base_url: "https://nearblocks.io",
            tx_path: TXNS_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}
