use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TRANSACTION_PATH, TXN_PATH, ACCOUNT_PATH, COIN_PATH};

pub fn new_aptos_scan() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "AptosScan",
        base_url: "https://aptoscan.com",
        tx_path: TRANSACTION_PATH,
        address_path: ACCOUNT_PATH,
        token_path: Some(COIN_PATH),
        validator_path: None,
    })
}

pub fn new_aptos_explorer() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "AptosExplorer",
        base_url: "https://explorer.aptoslabs.com",
        tx_path: TXN_PATH,
        address_path: ACCOUNT_PATH,
        token_path: Some(COIN_PATH),
        validator_path: None,
    })
}