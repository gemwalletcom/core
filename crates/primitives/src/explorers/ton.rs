use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ADDRESS_PATH, Explorer, Metadata, TRANSACTION_PATH, TX_PATH};

pub fn new_ton_viewer() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "TonViewer",
        base_url: "https://tonviewer.com",
        tx_path: TRANSACTION_PATH,
        address_path: "",
        token_path: Some(""),
        validator_path: Some(""),
    })
}

pub struct TonScan;

impl TonScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Tonscan",
            base_url: "https://tonscan.org",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some("/jetton"),
            validator_path: Some(ADDRESS_PATH),
        })
    }
}
