use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, ADDRESS_PATH, TRANSACTION_PATH};

pub struct TronScan;

impl TronScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "TRONSCAN",
            base_url: "https://tronscan.org/#",
            tx_path: TRANSACTION_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some("/token20"),
            validator_path: Some(ADDRESS_PATH),
        })
    }
}
