use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{GenericExplorer, Metadata};

pub struct BlockScout;

impl BlockScout {
    pub fn new_celo() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "BlockScout",
            base_url: "https://celo.blockscout.com",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }

    pub fn new_manta() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "Pacific Explorer",
            base_url: "https://pacific-explorer.manta.network",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }

    pub fn new_ink() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "Ink Explorer",
            base_url: "https://explorer.inkonchain.com",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }

    pub fn new_hyperliquid() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "BlockScout",
            base_url: "https://hyperliquid.cloud.blockscout.com",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }
}
