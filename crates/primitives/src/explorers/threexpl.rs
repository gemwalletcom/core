use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Metadata, MultiChainExplorer};
use std::sync::LazyLock;

static THREE_XPL_FACTORY: LazyLock<MultiChainExplorer> = LazyLock::new(|| {
    MultiChainExplorer::new()
        .add_chain("bitcoin", Metadata::blockchair("3xpl", "https://3xpl.com/bitcoin"))
        .add_chain("bitcoin_cash", Metadata::blockchair("3xpl", "https://3xpl.com/bitcoin-cash"))
        .add_chain("litecoin", Metadata::blockchair("3xpl", "https://3xpl.com/litecoin"))
        .add_chain("dogecoin", Metadata::blockchair("3xpl", "https://3xpl.com/dogecoin"))
        .add_chain("zcash", Metadata::blockchair("3xpl", "https://3xpl.com/zcash"))
});

pub fn new_bitcoin() -> Box<dyn BlockExplorer> {
    THREE_XPL_FACTORY.for_chain("bitcoin").unwrap()
}

pub fn new_bitcoin_cash() -> Box<dyn BlockExplorer> {
    THREE_XPL_FACTORY.for_chain("bitcoin_cash").unwrap()
}

pub fn new_litecoin() -> Box<dyn BlockExplorer> {
    THREE_XPL_FACTORY.for_chain("litecoin").unwrap()
}

pub fn new_doge() -> Box<dyn BlockExplorer> {
    THREE_XPL_FACTORY.for_chain("dogecoin").unwrap()
}

pub fn new_zcash() -> Box<dyn BlockExplorer> {
    THREE_XPL_FACTORY.for_chain("zcash").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_xpl_bitcoin() {
        let explorer = new_bitcoin();
        assert_eq!(explorer.name(), "3xpl");
        assert_eq!(explorer.get_tx_url("abc123"), "https://3xpl.com/bitcoin/transaction/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://3xpl.com/bitcoin/address/addr123");
    }

    #[test]
    fn test_three_xpl_litecoin() {
        let explorer = new_litecoin();
        assert_eq!(explorer.name(), "3xpl");
        assert_eq!(explorer.get_tx_url("abc123"), "https://3xpl.com/litecoin/transaction/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://3xpl.com/litecoin/address/addr123");
    }
}
