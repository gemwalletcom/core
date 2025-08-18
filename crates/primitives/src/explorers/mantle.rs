use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ADDRESS_PATH, TOKEN_PATH};

pub struct MantleExplorer;

impl MantleExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Mantle Explorer",
            base_url: "https://explorer.mantle.xyz",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }
}
