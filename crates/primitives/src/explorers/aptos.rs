use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{GenericExplorer, Metadata};

pub fn new_aptos_scan() -> Box<dyn BlockExplorer> {
    GenericExplorer::new(Metadata {
        name: "AptosScan",
        base_url: "https://aptoscan.com",
        tx_path: "transaction",
        address_path: "account",
        token_path: Some("coin"),
        validator_path: None,
    })
}

pub fn new_aptos_explorer() -> Box<dyn BlockExplorer> {
    GenericExplorer::new(Metadata {
        name: "AptosExplorer",
        base_url: "https://explorer.aptoslabs.com",
        tx_path: "txn",
        address_path: "account",
        token_path: Some("coin"),
        validator_path: None,
    })
}