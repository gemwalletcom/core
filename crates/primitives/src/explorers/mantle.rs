use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{GenericExplorer, Metadata};

pub struct MantleExplorer;

impl MantleExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "Mantle Explorer",
            base_url: "https://explorer.mantle.xyz",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }
}
