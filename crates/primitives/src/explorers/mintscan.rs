use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Metadata, MultiChainExplorer, TX_PATH, ADDRESS_PATH, VALIDATORS_PATH, ASSETS_PATH};
use std::sync::LazyLock;

static MINTSCAN_FACTORY: LazyLock<MultiChainExplorer> = LazyLock::new(|| {
    MultiChainExplorer::new()
        .add_chain(
            "cosmos",
            Metadata {
                name: "Mintscan",
                base_url: "https://www.mintscan.io/cosmos",
                tx_path: TX_PATH,
                address_path: ADDRESS_PATH,
                token_path: Some(ASSETS_PATH),
                validator_path: Some(VALIDATORS_PATH),
            },
        )
        .add_chain(
            "osmosis",
            Metadata {
                name: "Mintscan",
                base_url: "https://www.mintscan.io/osmosis",
                tx_path: TX_PATH,
                address_path: ADDRESS_PATH,
                token_path: Some(ASSETS_PATH),
                validator_path: Some(VALIDATORS_PATH),
            },
        )
        .add_chain(
            "celestia",
            Metadata {
                name: "Mintscan",
                base_url: "https://www.mintscan.io/celestia",
                tx_path: TX_PATH,
                address_path: ADDRESS_PATH,
                token_path: Some(ASSETS_PATH),
                validator_path: Some(VALIDATORS_PATH),
            },
        )
        .add_chain(
            "injective",
            Metadata {
                name: "Mintscan",
                base_url: "https://www.mintscan.io/injective-protocol",
                tx_path: TX_PATH,
                address_path: ADDRESS_PATH,
                token_path: Some(ASSETS_PATH),
                validator_path: Some(VALIDATORS_PATH),
            },
        )
        .add_chain(
            "sei",
            Metadata {
                name: "Mintscan",
                base_url: "https://www.mintscan.io/sei",
                tx_path: TX_PATH,
                address_path: ADDRESS_PATH,
                token_path: Some(ASSETS_PATH),
                validator_path: Some(VALIDATORS_PATH),
            },
        )
        .add_chain(
            "noble",
            Metadata {
                name: "Mintscan",
                base_url: "https://www.mintscan.io/noble",
                tx_path: TX_PATH,
                address_path: ADDRESS_PATH,
                token_path: Some(ASSETS_PATH),
                validator_path: Some(VALIDATORS_PATH),
            },
        )
});

pub fn new_cosmos() -> Box<dyn BlockExplorer> {
    MINTSCAN_FACTORY.for_chain("cosmos").unwrap()
}

pub fn new_osmosis() -> Box<dyn BlockExplorer> {
    MINTSCAN_FACTORY.for_chain("osmosis").unwrap()
}

pub fn new_celestia() -> Box<dyn BlockExplorer> {
    MINTSCAN_FACTORY.for_chain("celestia").unwrap()
}

pub fn new_injective() -> Box<dyn BlockExplorer> {
    MINTSCAN_FACTORY.for_chain("injective").unwrap()
}

pub fn new_sei() -> Box<dyn BlockExplorer> {
    MINTSCAN_FACTORY.for_chain("sei").unwrap()
}

pub fn new_noble() -> Box<dyn BlockExplorer> {
    MINTSCAN_FACTORY.for_chain("noble").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mintscan_cosmos() {
        let explorer = new_cosmos();
        assert_eq!(explorer.name(), "Mintscan");
        assert_eq!(explorer.get_tx_url("abc123"), "https://www.mintscan.io/cosmos/tx/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://www.mintscan.io/cosmos/address/addr123");
        assert_eq!(explorer.get_validator_url("val123"), Some("https://www.mintscan.io/cosmos/validators/val123".to_string()));
    }

    #[test]
    fn test_mintscan_osmosis() {
        let explorer = new_osmosis();
        assert_eq!(explorer.name(), "Mintscan");
        assert_eq!(explorer.get_tx_url("abc123"), "https://www.mintscan.io/osmosis/tx/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://www.mintscan.io/osmosis/address/addr123");
        assert_eq!(explorer.get_validator_url("val123"), Some("https://www.mintscan.io/osmosis/validators/val123".to_string()));
    }
}