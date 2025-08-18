use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ADDRESS_PATH, TOKEN_PATH};
use crate::block_explorer::BlockExplorer;

pub struct BlockScout;

impl BlockScout {
    pub fn new_celo() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "BlockScout",
            base_url: "https://celo.blockscout.com",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }

    pub fn new_manta() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Pacific Explorer",
            base_url: "https://pacific-explorer.manta.network",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }

    pub fn new_ink() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Ink Explorer",
            base_url: "https://explorer.inkonchain.com",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }

    pub fn new_hyperliquid() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "BlockScout",
            base_url: "https://hyperliquid.cloud.blockscout.com",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }
}
