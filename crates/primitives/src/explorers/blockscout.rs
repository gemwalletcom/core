use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct BlockScout;

impl BlockScout {
    pub fn new_celo() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("BlockScout", "https://celo.blockscout.com"))
    }

    pub fn new_manta() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("Pacific Explorer", "https://pacific-explorer.manta.network"))
    }

    pub fn new_ink() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("Ink Explorer", "https://explorer.inkonchain.com"))
    }

    pub fn new_hyperliquid() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("BlockScout", "https://hyperliquid.cloud.blockscout.com"))
    }
}
