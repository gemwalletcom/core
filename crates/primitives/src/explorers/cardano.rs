use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TRANSACTION_PATH, ADDRESS_PATH};

pub struct Cardanocan;

impl Cardanocan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "CardanoScan",
            base_url: "https://cardanoscan.io",
            tx_path: TRANSACTION_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}
