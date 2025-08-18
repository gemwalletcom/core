use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{GenericExplorer, Metadata};

pub struct RouteScan;

impl RouteScan {
    pub fn new_avax() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "SnowTrace",
            base_url: "https://snowtrace.io",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }

    pub fn new_sonic() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "RouteScan",
            base_url: "https://146.routescan.io",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }

    pub fn new_ink() -> Box<dyn BlockExplorer> {
        GenericExplorer::new(Metadata {
            name: "RouteScan",
            base_url: "https://57073.routescan.io",
            tx_path: "tx",
            address_path: "address",
            token_path: Some("token"),
            validator_path: None,
        })
    }
}
