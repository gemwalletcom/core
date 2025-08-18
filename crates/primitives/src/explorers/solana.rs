use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{GenericExplorer, Metadata};

pub fn new_solscan() -> Box<dyn BlockExplorer> {
    GenericExplorer::new(Metadata {
        name: "Solscan",
        base_url: "https://solscan.io",
        tx_path: "tx",
        address_path: "account",
        token_path: Some("token"),
        validator_path: None,
    })
}

pub fn new_solana_fm() -> Box<dyn BlockExplorer> {
    GenericExplorer::new(Metadata {
        name: "SolanaFM",
        base_url: "https://solana.fm",
        tx_path: "tx",
        address_path: "address",
        token_path: Some("address"),
        validator_path: None,
    })
}