use super::metadata::{Explorer, Metadata, ACCOUNT_PATH, TX_PATH};
use crate::block_explorer::BlockExplorer;

pub struct XrpScan;

impl XrpScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        let config = Metadata {
            name: "XrpScan",
            base_url: "https://xrpscan.com",
            tx_path: TX_PATH,
            address_path: ACCOUNT_PATH,
            token_path: Some(ACCOUNT_PATH),
            validator_path: None,
        };
        Explorer::boxed(config)
    }
}
