use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ADDRESS_PATH, Explorer, Metadata, TXNS_PATH};

pub const NEAR_BLOCKS_BASE_URL: &str = "https://nearblocks.io";

pub struct NearBlocks;

impl NearBlocks {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Near",
            base_url: NEAR_BLOCKS_BASE_URL,
            tx_path: TXNS_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}
