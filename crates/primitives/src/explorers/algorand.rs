use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, Explorer, Metadata, TX_PATH};

pub struct AlgorandAllo;

impl AlgorandAllo {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Allo",
            base_url: "https://allo.info",
            tx_path: TX_PATH,
            address_path: ACCOUNT_PATH,
            token_path: None,
            nft_path: None,
            validator_path: Some(ACCOUNT_PATH),
        })
    }
}

pub struct AlgorandPera;

impl AlgorandPera {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Pera",
            base_url: "https://explorer.perawallet.app",
            tx_path: TX_PATH,
            address_path: ACCOUNT_PATH,
            token_path: Some("/assets"),
            nft_path: None,
            validator_path: Some(ACCOUNT_PATH),
        })
    }
}
