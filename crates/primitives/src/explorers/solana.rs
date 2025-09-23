use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, ADDRESS_PATH, Explorer, Metadata, TOKEN_PATH, TX_PATH};

pub fn new_solscan() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "Solscan",
        base_url: "https://solscan.io",
        tx_path: TX_PATH,
        address_path: ACCOUNT_PATH,
        token_path: Some(TOKEN_PATH),
        validator_path: None,
    })
}

pub fn new_solana_fm() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "SolanaFM",
        base_url: "https://solana.fm",
        tx_path: TX_PATH,
        address_path: ADDRESS_PATH,
        token_path: Some(ADDRESS_PATH),
        validator_path: None,
    })
}
