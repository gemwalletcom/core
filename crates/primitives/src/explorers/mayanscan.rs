use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, ADDRESS_PATH, TX_PATH};

pub struct MayanScan;

impl MayanScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Mayan Explorer",
            base_url: "https://explorer.mayan.finance",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}
