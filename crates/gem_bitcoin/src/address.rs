use bitcoin::ScriptBuf;
use primitives::{Address as AddressTrait, BitcoinChain, Chain};

use crate::signer::address::script_for_address;

#[derive(Debug, Clone)]
pub struct BitcoinAddress {
    chain: BitcoinChain,
    address: String,
    script_pubkey: ScriptBuf,
}

impl BitcoinAddress {
    pub fn try_parse_for_chain(address: &str, chain: BitcoinChain) -> Option<Self> {
        let script_pubkey = script_for_address(chain, address).ok()?.script_pubkey;
        Some(Self {
            chain,
            address: address.to_string(),
            script_pubkey,
        })
    }

    pub fn is_valid_for_chain(address: &str, chain: Chain) -> bool {
        BitcoinChain::from_chain(chain).is_some_and(|chain| Self::try_parse_for_chain(address, chain).is_some())
    }

    pub fn bitcoin_chain(&self) -> BitcoinChain {
        self.chain
    }
}

impl AddressTrait for BitcoinAddress {
    fn try_parse(address: &str) -> Option<Self> {
        [
            BitcoinChain::Bitcoin,
            BitcoinChain::BitcoinCash,
            BitcoinChain::Litecoin,
            BitcoinChain::Doge,
            BitcoinChain::Zcash,
        ]
        .into_iter()
        .find_map(|chain| Self::try_parse_for_chain(address, chain))
    }

    fn as_bytes(&self) -> &[u8] {
        self.script_pubkey.as_bytes()
    }

    fn encode(&self) -> String {
        self.address.clone()
    }
}

pub fn validate_address(address: &str, chain: Chain) -> bool {
    BitcoinAddress::is_valid_for_chain(address, chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Address as AddressTrait;

    #[test]
    fn test_validate_address() {
        let bitcoin = BitcoinAddress::mock();
        let bitcoin_cash = BitcoinAddress::mock_with_chain(BitcoinChain::BitcoinCash);
        let litecoin = BitcoinAddress::mock_with_chain(BitcoinChain::Litecoin);
        let doge = BitcoinAddress::mock_with_chain(BitcoinChain::Doge);
        let zcash = BitcoinAddress::mock_with_chain(BitcoinChain::Zcash);

        assert!(validate_address(&bitcoin.encode(), Chain::Bitcoin));
        assert!(validate_address(&bitcoin_cash.encode(), Chain::BitcoinCash));
        assert!(validate_address(bitcoin_cash.encode().strip_prefix("bitcoincash:").unwrap(), Chain::BitcoinCash));
        assert!(validate_address(&litecoin.encode(), Chain::Litecoin));
        assert!(validate_address(&doge.encode(), Chain::Doge));
        assert!(validate_address(&zcash.encode(), Chain::Zcash));
        assert!(!validate_address(&bitcoin.encode(), Chain::Litecoin));
        assert!(!validate_address("invalid", Chain::Bitcoin));

        let parsed = BitcoinAddress::try_parse_for_chain(&bitcoin.encode(), BitcoinChain::Bitcoin).unwrap();
        assert_eq!(parsed.bitcoin_chain().get_chain(), Chain::Bitcoin);
        assert_eq!(parsed.encode(), bitcoin.encode());
        assert_eq!(hex::encode(parsed.as_bytes()), "0014751e76e8199196d454941c45d1b3a323f1433bd6");
    }
}
