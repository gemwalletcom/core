mod bitcoin;
mod bitcoin_cash;
mod doge;
mod litecoin;
mod script;
mod zcash;

use primitives::{BitcoinChain, SignerError};

pub(crate) use crate::hash::public_key_hash;
use script::AddressScript;
pub(crate) use script::{UnlockingScript, script_for_public_key_hash};
#[cfg(test)]
pub(crate) use zcash::TRANSPARENT_P2PKH_PREFIX as ZCASH_TRANSPARENT_P2PKH_PREFIX;

pub(crate) fn script_for_address(chain: BitcoinChain, address: &str) -> Result<AddressScript, SignerError> {
    match chain {
        BitcoinChain::Bitcoin => bitcoin::script(address),
        BitcoinChain::Litecoin => litecoin::script(address),
        BitcoinChain::Doge => doge::script(address),
        BitcoinChain::BitcoinCash => bitcoin_cash::script(address),
        BitcoinChain::Zcash => zcash::script(address),
    }
}

#[cfg(test)]
mod tests {
    use primitives::BitcoinChain;

    use super::{
        script::{LockingScript, UnlockingScript},
        script_for_address,
    };

    fn assert_address_script(
        chain: BitcoinChain,
        address: &str,
        locking_script: LockingScript,
        unlocking_script: Option<UnlockingScript>,
        script_pubkey: &str,
        public_key_hash: Option<&str>,
    ) {
        let script = script_for_address(chain, address).unwrap();
        assert_eq!(script.locking_script, locking_script);
        assert_eq!(script.unlocking_script(), unlocking_script);
        assert_eq!(hex::encode(script.script_pubkey.as_bytes()), script_pubkey);
        assert_eq!(script.public_key_hash().map(hex::encode).as_deref(), public_key_hash);
    }

    #[test]
    fn test_script_for_address_bitcoin() {
        assert_address_script(
            BitcoinChain::Bitcoin,
            "1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY",
            LockingScript::P2pkh,
            Some(UnlockingScript::P2pkh),
            "76a914ff99864ce1a887e00c9c8615210d6267edd7d7a588ac",
            Some("ff99864ce1a887e00c9c8615210d6267edd7d7a5"),
        );
        assert_address_script(
            BitcoinChain::Bitcoin,
            "33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k",
            LockingScript::P2sh,
            None,
            "a914162c5ea71c0b23f5b9022ef047c4a86470a5b07087",
            None,
        );
        assert_address_script(
            BitcoinChain::Bitcoin,
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            LockingScript::P2wpkh,
            Some(UnlockingScript::P2wpkh),
            "0014751e76e8199196d454941c45d1b3a323f1433bd6",
            Some("751e76e8199196d454941c45d1b3a323f1433bd6"),
        );
        assert_address_script(
            BitcoinChain::Bitcoin,
            "bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej",
            LockingScript::P2wsh,
            None,
            "0020701a8d401c84fb13e6baf169d59684e17abd9fa216c8cc5b9fc63d622ff8c58d",
            None,
        );
        assert_address_script(
            BitcoinChain::Bitcoin,
            "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr",
            LockingScript::P2tr,
            None,
            "5120a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6880949dc684c",
            None,
        );
    }

    #[test]
    fn test_script_for_address_bitcoin_cash() {
        assert_address_script(
            BitcoinChain::BitcoinCash,
            "bitcoincash:qp3wjpa3tjlj042z2wv7hahsldgwhwy0rq9sywjpyy",
            LockingScript::P2pkh,
            Some(UnlockingScript::P2pkh),
            "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
            Some("62e907b15cbf27d5425399ebf6f0fb50ebb88f18"),
        );
        assert_address_script(
            BitcoinChain::BitcoinCash,
            "qp3wjpa3tjlj042z2wv7hahsldgwhwy0rq9sywjpyy",
            LockingScript::P2pkh,
            Some(UnlockingScript::P2pkh),
            "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
            Some("62e907b15cbf27d5425399ebf6f0fb50ebb88f18"),
        );

        assert_address_script(
            BitcoinChain::BitcoinCash,
            "bitcoincash:pr0662zpd7vr936d83f64u629v886aan7c77r3j5v5",
            LockingScript::P2sh,
            None,
            "a914dfad28416f9832c74d3c53aaf34a2b0e7d77b3f687",
            None,
        );
    }

    #[test]
    fn test_script_for_address_litecoin() {
        assert_address_script(
            BitcoinChain::Litecoin,
            "LMHEFMwRsQ3nHDfb9zZqynLHxjuJ2hgyyW",
            LockingScript::P2pkh,
            Some(UnlockingScript::P2pkh),
            "76a914168ed7e47426cf09541df4979c6450b3d5a5547088ac",
            Some("168ed7e47426cf09541df4979c6450b3d5a55470"),
        );
        assert_address_script(
            BitcoinChain::Litecoin,
            "MC2JYMPVWaxqUb9qUkUbjtUwoNMo1tPaLF",
            LockingScript::P2sh,
            None,
            "a9142d3a59d2d9f68868cbd5d37afb2c0d6c921b2f3187",
            None,
        );
        assert_address_script(
            BitcoinChain::Litecoin,
            "ltc1qhzjptwpym9afcdjhs7jcz6fd0jma0l0rc0e5yr",
            LockingScript::P2wpkh,
            Some(UnlockingScript::P2wpkh),
            "0014b8a415b824d97a9c365787a581692d7cb7d7fde3",
            Some("b8a415b824d97a9c365787a581692d7cb7d7fde3"),
        );
    }

    #[test]
    fn test_script_for_address_doge() {
        assert_address_script(
            BitcoinChain::Doge,
            "DMKhUaRmnxJXfDxyFguMnMjVdgvnNipFzt",
            LockingScript::P2pkh,
            Some(UnlockingScript::P2pkh),
            "76a914b18355f0b9c7aa20e9db204825e6275e9a40bc8988ac",
            Some("b18355f0b9c7aa20e9db204825e6275e9a40bc89"),
        );
        assert_address_script(
            BitcoinChain::Doge,
            "A1yb6viUzAcUWftRHT6GpnCwvhXHg4CV1x",
            LockingScript::P2sh,
            None,
            "a91468a56b88a61df17afc8a0709ec1536a51101881087",
            None,
        );
    }

    #[test]
    fn test_script_for_address_zcash() {
        assert_address_script(
            BitcoinChain::Zcash,
            "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML",
            LockingScript::P2pkh,
            Some(UnlockingScript::P2pkh),
            "76a9141634f5ff0b8f6603a17570436d6c12a91f4b1fed88ac",
            Some("1634f5ff0b8f6603a17570436d6c12a91f4b1fed"),
        );
        assert_address_script(
            BitcoinChain::Zcash,
            "t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd",
            LockingScript::P2sh,
            None,
            "a9147d46a730d31f97b1930d3368a967c309bd4d136a87",
            None,
        );
    }
}
