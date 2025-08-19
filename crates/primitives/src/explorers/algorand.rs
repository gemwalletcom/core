use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ACCOUNT_PATH};

pub struct AlgorandAllo;

impl AlgorandAllo {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Allo",
            base_url: "https://allo.info",
            tx_path: TX_PATH,
            address_path: ACCOUNT_PATH,
            token_path: None,
            validator_path: Some(ACCOUNT_PATH),
        })
    }
}
