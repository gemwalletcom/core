use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{GenericExplorer, Metadata};

pub struct Cardanocan;

impl Cardanocan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "CardanoScan",
            base_url: "https://cardanoscan.io",
            tx_path: "transaction",
            address_path: "address",
            token_path: None,
            validator_path: None,
        })
    }
}
