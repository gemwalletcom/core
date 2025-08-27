use crate::block_explorer::BlockExplorer;
use super::metadata::{Explorer, Metadata, ADDRESS_PATH, TX_PATH};

pub struct ZkSync;

impl ZkSync {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        let config = Metadata {
            name: "zkSync.io",
            base_url: "https://explorer.zksync.io",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(ADDRESS_PATH), // ZkSync uses address path for tokens
            validator_path: None,
        };
        Explorer::boxed(config)
    }
}
