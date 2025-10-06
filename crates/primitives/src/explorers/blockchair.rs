use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Metadata, MultiChainExplorer};
use std::sync::LazyLock;

static BLOCKCHAIR_FACTORY: LazyLock<MultiChainExplorer> = LazyLock::new(|| {
    MultiChainExplorer::new()
        .add_chain("bitcoin", Metadata::blockchair("Blockchair", "https://blockchair.com/bitcoin"))
        .add_chain("bitcoin_cash", Metadata::blockchair("Blockchair", "https://blockchair.com/bitcoin-cash"))
        .add_chain("litecoin", Metadata::blockchair("Blockchair", "https://blockchair.com/litecoin"))
        .add_chain("dogecoin", Metadata::blockchair("Blockchair", "https://blockchair.com/dogecoin"))
        .add_chain("zcash", Metadata::blockchair("Blockchair", "https://blockchair.com/zcash"))
        .add_chain("ethereum", Metadata::blockchair("Blockchair", "https://blockchair.com/ethereum"))
        .add_chain("base", Metadata::blockchair("Blockchair", "https://blockchair.com/base"))
        .add_chain("polygon", Metadata::blockchair("Blockchair", "https://blockchair.com/polygon"))
        .add_chain("arbitrum", Metadata::blockchair("Blockchair", "https://blockchair.com/arbitrum-one"))
        .add_chain("optimism", Metadata::blockchair("Blockchair", "https://blockchair.com/optimism"))
        .add_chain("avalanche", Metadata::blockchair("Blockchair", "https://blockchair.com/avalanche"))
        .add_chain("solana", Metadata::blockchair("Blockchair", "https://blockchair.com/solana"))
        .add_chain("stellar", Metadata::blockchair("Blockchair", "https://blockchair.com/stellar"))
        .add_chain("bnb", Metadata::blockchair("Blockchair", "https://blockchair.com/bnb"))
        .add_chain("opbnb", Metadata::blockchair("Blockchair", "https://blockchair.com/opbnb"))
        .add_chain("fantom", Metadata::blockchair("Blockchair", "https://blockchair.com/fantom"))
        .add_chain("gnosis", Metadata::blockchair("Blockchair", "https://blockchair.com/gnosis-chain"))
        .add_chain("linea", Metadata::blockchair("Blockchair", "https://blockchair.com/linea"))
        .add_chain("ton", Metadata::blockchair("Blockchair", "https://blockchair.com/ton"))
        .add_chain("tron", Metadata::blockchair("Blockchair", "https://blockchair.com/tron"))
        .add_chain("xrp", Metadata::blockchair("Blockchair", "https://blockchair.com/xrp-ledger"))
        .add_chain("aptos", Metadata::blockchair("Blockchair", "https://blockchair.com/aptos"))
        .add_chain("polkadot", Metadata::blockchair("Blockchair", "https://blockchair.com/polkadot"))
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

pub fn new_zcash() -> Box<dyn BlockExplorer> {
    BLOCKCHAIR_FACTORY.for_chain("zcash").unwrap()
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
