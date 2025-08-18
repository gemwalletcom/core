use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Metadata, MultiChainExplorer, TRANSACTION_PATH, ADDRESS_PATH};
use std::sync::LazyLock;

static BLOCKCHAIR_FACTORY: LazyLock<MultiChainExplorer> = LazyLock::new(|| {
    MultiChainExplorer::new()
        .add_chain(
            "bitcoin",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/bitcoin",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "bitcoin_cash",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/bitcoin-cash",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "litecoin",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/litecoin",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "dogecoin",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/dogecoin",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "ethereum",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/ethereum",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "base",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/base",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "polygon",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/polygon",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "arbitrum",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/arbitrum-one",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "optimism",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/optimism",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "avalanche",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/avalanche",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "solana",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/solana",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "stellar",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/stellar",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "bnb",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/bnb",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "opbnb",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/opbnb",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "fantom",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/fantom",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "gnosis",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/gnosis-chain",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "linea",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/linea",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "ton",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/ton",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "tron",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/tron",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "xrp",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/xrp-ledger",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "aptos",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/aptos",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
        .add_chain(
            "polkadot",
            Metadata {
                name: "Blockchair",
                base_url: "https://blockchair.com/polkadot",
                tx_path: TRANSACTION_PATH,
                address_path: ADDRESS_PATH,
                token_path: None,
                validator_path: None,
            },
        )
});

pub fn new_bitcoin() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("bitcoin").unwrap()
}

pub fn new_bitcoin_cash() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("bitcoin_cash").unwrap()
}

pub fn new_litecoin() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("litecoin").unwrap()
}

pub fn new_doge() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("dogecoin").unwrap()
}

pub fn new_ethereum() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("ethereum").unwrap()
}

pub fn new_base() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("base").unwrap()
}

pub fn new_polygon() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("polygon").unwrap()
}

pub fn new_arbitrum() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("arbitrum").unwrap()
}

pub fn new_optimism() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("optimism").unwrap()
}

pub fn new_avalanche() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("avalanche").unwrap()
}

pub fn new_solana() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("solana").unwrap()
}

pub fn new_stellar() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("stellar").unwrap()
}

pub fn new_bnb() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("bnb").unwrap()
}

pub fn new_opbnb() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("opbnb").unwrap()
}

pub fn new_fantom() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("fantom").unwrap()
}

pub fn new_gnosis() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("gnosis").unwrap()
}

pub fn new_linea() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("linea").unwrap()
}

pub fn new_ton() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("ton").unwrap()
}

pub fn new_tron() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("tron").unwrap()
}

pub fn new_xrp() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("xrp").unwrap()
}

pub fn new_aptos() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("aptos").unwrap()
}

pub fn new_polkadot() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("polkadot").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchair_bitcoin() {
        let explorer = new_bitcoin();
        assert_eq!(explorer.name(), "Blockchair");
        assert_eq!(explorer.get_tx_url("abc123"), "https://blockchair.com/bitcoin/transaction/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://blockchair.com/bitcoin/address/addr123");
    }

    #[test]
    fn test_blockchair_ethereum() {
        let explorer = new_ethereum();
        assert_eq!(explorer.name(), "Blockchair");
        assert_eq!(explorer.get_tx_url("abc123"), "https://blockchair.com/ethereum/transaction/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://blockchair.com/ethereum/address/addr123");
    }

    #[test]
    fn test_blockchair_stellar() {
        let explorer = new_stellar();
        assert_eq!(explorer.name(), "Blockchair");
        assert_eq!(explorer.get_tx_url("abc123"), "https://blockchair.com/stellar/transaction/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://blockchair.com/stellar/address/addr123");
    }
}