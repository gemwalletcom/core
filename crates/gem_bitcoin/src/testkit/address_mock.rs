use primitives::BitcoinChain;

use crate::{address::BitcoinAddress, signer::address::ZCASH_TRANSPARENT_P2PKH_PREFIX};

pub(crate) const TEST_BITCOIN_P2WPKH_ADDRESS: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
pub(crate) const TEST_BITCOIN_P2WPKH_HASH: [u8; 20] = [
    0x75, 0x1e, 0x76, 0xe8, 0x19, 0x91, 0x96, 0xd4, 0x54, 0x94, 0x1c, 0x45, 0xd1, 0xb3, 0xa3, 0x23, 0xf1, 0x43, 0x3b, 0xd6,
];

impl BitcoinAddress {
    pub fn mock() -> Self {
        Self::mock_with_chain(BitcoinChain::Bitcoin)
    }

    pub fn mock_with_chain(chain: BitcoinChain) -> Self {
        Self::try_parse_for_chain(&Self::mock_address_with_chain(chain), chain).unwrap()
    }

    pub fn mock_address_with_chain(chain: BitcoinChain) -> String {
        match chain {
            BitcoinChain::Bitcoin => TEST_BITCOIN_P2WPKH_ADDRESS.to_string(),
            BitcoinChain::BitcoinCash => address_for_hash(chain, [2u8; 20]),
            BitcoinChain::Litecoin => address_for_hash(chain, [3u8; 20]),
            BitcoinChain::Doge => address_for_hash(chain, [4u8; 20]),
            BitcoinChain::Zcash => address_for_hash(chain, [5u8; 20]),
        }
    }
}

pub(crate) fn address_for_hash(chain: BitcoinChain, hash: [u8; 20]) -> String {
    match chain {
        BitcoinChain::Bitcoin => prefixed_address(&[0], hash),
        BitcoinChain::BitcoinCash => bitcoin_cash_address(hash),
        BitcoinChain::Litecoin => prefixed_address(&[48], hash),
        BitcoinChain::Doge => prefixed_address(&[30], hash),
        BitcoinChain::Zcash => zcash_address(hash),
    }
}

pub(crate) fn bitcoin_cash_address(hash: [u8; 20]) -> String {
    bitcoincash_addr::Address::new(
        hash.to_vec(),
        bitcoincash_addr::Scheme::CashAddr,
        bitcoincash_addr::HashType::Key,
        bitcoincash_addr::Network::Main,
    )
    .encode()
    .unwrap()
}

pub(crate) fn zcash_address(hash: [u8; 20]) -> String {
    prefixed_address(&ZCASH_TRANSPARENT_P2PKH_PREFIX, hash)
}

pub(crate) fn prefixed_address(prefix: &[u8], hash: [u8; 20]) -> String {
    let mut payload = prefix.to_vec();
    payload.extend(hash);
    bs58::encode(payload).with_check().into_string()
}
