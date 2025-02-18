use crate::block_explorer::{BlockExplorer, Metadata};
use crate::chain_evm::EVMChain;

use super::EtherScan;

static BLOCKSEC_NAME: &str = "Blocksec Phalcon";

pub struct Blocksec {
    pub meta: Metadata,
    pub chain: EVMChain,
    pub tx_suffix: Option<&'static str>,
}

impl Blocksec {
    pub fn new(chain: EVMChain, tx_suffix: Option<&'static str>) -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: BLOCKSEC_NAME,
                base_url: "https://app.blocksec.com/explorer/tx",
            },
            chain,
            tx_suffix,
        })
    }

    pub fn new_ethereum() -> Box<Self> {
        Self::new(EVMChain::Ethereum, Some("eth"))
    }

    pub fn new_bsc() -> Box<Self> {
        Self::new(EVMChain::SmartChain, Some("bsc"))
    }

    pub fn new_polygon() -> Box<Self> {
        Self::new(EVMChain::Polygon, None)
    }

    pub fn new_arbitrum() -> Box<Self> {
        Self::new(EVMChain::Arbitrum, None)
    }

    pub fn new_optimism() -> Box<Self> {
        Self::new(EVMChain::Optimism, None)
    }
    pub fn new_base() -> Box<Self> {
        Self::new(EVMChain::Base, None)
    }
}

impl BlockExplorer for Blocksec {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/{}/{}", self.meta.base_url, self.tx_suffix.unwrap_or_else(|| self.chain.as_ref()), hash)
    }
    fn get_address_url(&self, _address: &str) -> String {
        // delegate to etherscan
        EtherScan::new(self.chain).get_address_url(_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_ethereum_tx_url() {
        let explorer = Blocksec::new_ethereum();
        assert_eq!(
            explorer.get_tx_url("0x08729ee8d311df87fe9ad5f73e60d8740b7688f19932ecbe8ebe3fac8321284e"),
            "https://app.blocksec.com/explorer/tx/eth/0x08729ee8d311df87fe9ad5f73e60d8740b7688f19932ecbe8ebe3fac8321284e"
        );
    }

    #[test]
    fn test_get_optimism_tx_url() {
        let explorer = Blocksec::new_optimism();
        assert_eq!(
            explorer.get_tx_url("0x4a81ba47adfb9720f792eb08cef9a4d444db7f6ff574c9adc4870188acb1cb18"),
            "https://app.blocksec.com/explorer/tx/optimism/0x4a81ba47adfb9720f792eb08cef9a4d444db7f6ff574c9adc4870188acb1cb18"
        );
    }

    #[test]
    fn test_get_bsc_tx_url() {
        let explorer = Blocksec::new_bsc();

        assert_eq!(
            explorer.get_tx_url("0xa9fe9d47f5130e3aa622b1f1c9a7af04f68a297a126e9210c671b3afb5df2816"),
            "https://app.blocksec.com/explorer/tx/bsc/0xa9fe9d47f5130e3aa622b1f1c9a7af04f68a297a126e9210c671b3afb5df2816"
        );
        assert_eq!(
            explorer.get_address_url("0xba4d1d35bce0e8f28e5a3403e7a0b996c5d50ac4"),
            "https://bscscan.com/address/0xba4d1d35bce0e8f28e5a3403e7a0b996c5d50ac4"
        )
    }

    #[test]
    fn test_get_base_url() {
        let explorer = Blocksec::new_base();

        assert_eq!(
            explorer.get_tx_url("0xa9fe9d47f5130e3aa622b1f1c9a7af04f68a297a126e9210c671b3afb5df2816"),
            "https://app.blocksec.com/explorer/tx/base/0xa9fe9d47f5130e3aa622b1f1c9a7af04f68a297a126e9210c671b3afb5df2816"
        );
    }
}
