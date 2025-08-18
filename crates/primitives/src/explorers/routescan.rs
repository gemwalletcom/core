use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ADDRESS_PATH, TOKEN_PATH};

pub struct RouteScan;

impl RouteScan {
    pub fn new_avax() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "SnowTrace",
            base_url: "https://snowtrace.io",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }

    pub fn new_sonic() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "RouteScan",
            base_url: "https://146.routescan.io",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }

    pub fn new_ink() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "RouteScan",
            base_url: "https://57073.routescan.io",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }
}
