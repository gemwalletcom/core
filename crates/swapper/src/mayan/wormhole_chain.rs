use crate::SwapperError;
use primitives::Chain;
use std::str::FromStr;
use strum::{EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub(in crate::mayan) enum WormholeChain {
    Solana,
    Ethereum,
    Bsc,
    Polygon,
    Avalanche,
    Fantom,
    Ton,
    Celo,
    Near,
    Sui,
    Aptos,
    Arbitrum,
    Optimism,
    Base,
    Linea,
    Berachain,
    Unichain,
    World,
    Hyperevm,
    Monad,
    Sonic,
    Plasma,
    Hypercore,
}

impl WormholeChain {
    pub(in crate::mayan) fn from_name(name: &str) -> Result<Self, SwapperError> {
        Self::from_str(name).map_err(|_| SwapperError::NotSupportedChain)
    }

    fn from_id(chain_id: u16) -> Option<Self> {
        match chain_id {
            1 => Some(Self::Solana),
            2 => Some(Self::Ethereum),
            4 => Some(Self::Bsc),
            5 => Some(Self::Polygon),
            6 => Some(Self::Avalanche),
            10 => Some(Self::Fantom),
            13 => Some(Self::Ton),
            14 => Some(Self::Celo),
            15 => Some(Self::Near),
            21 => Some(Self::Sui),
            22 => Some(Self::Aptos),
            23 => Some(Self::Arbitrum),
            24 => Some(Self::Optimism),
            30 => Some(Self::Base),
            38 => Some(Self::Linea),
            39 => Some(Self::Berachain),
            44 => Some(Self::Unichain),
            45 => Some(Self::World),
            47 => Some(Self::Hyperevm),
            48 => Some(Self::Monad),
            52 => Some(Self::Sonic),
            58 => Some(Self::Plasma),
            65000 => Some(Self::Hypercore),
            _ => None,
        }
    }

    fn from_chain(chain: Chain) -> Option<Self> {
        match chain {
            Chain::Solana => Some(Self::Solana),
            Chain::Ethereum => Some(Self::Ethereum),
            Chain::SmartChain => Some(Self::Bsc),
            Chain::Polygon => Some(Self::Polygon),
            Chain::AvalancheC => Some(Self::Avalanche),
            Chain::Fantom => Some(Self::Fantom),
            Chain::Ton => Some(Self::Ton),
            Chain::Celo => Some(Self::Celo),
            Chain::Near => Some(Self::Near),
            Chain::Sui => Some(Self::Sui),
            Chain::Aptos => Some(Self::Aptos),
            Chain::Arbitrum => Some(Self::Arbitrum),
            Chain::Optimism => Some(Self::Optimism),
            Chain::Base => Some(Self::Base),
            Chain::Linea => Some(Self::Linea),
            Chain::Berachain => Some(Self::Berachain),
            Chain::Unichain => Some(Self::Unichain),
            Chain::World => Some(Self::World),
            Chain::Hyperliquid => Some(Self::Hyperevm),
            Chain::Monad => Some(Self::Monad),
            Chain::Sonic => Some(Self::Sonic),
            Chain::Plasma => Some(Self::Plasma),
            Chain::HyperCore => Some(Self::Hypercore),
            _ => None,
        }
    }

    pub(in crate::mayan) const fn id(self) -> u16 {
        match self {
            Self::Solana => 1,
            Self::Ethereum => 2,
            Self::Bsc => 4,
            Self::Polygon => 5,
            Self::Avalanche => 6,
            Self::Fantom => 10,
            Self::Ton => 13,
            Self::Celo => 14,
            Self::Near => 15,
            Self::Sui => 21,
            Self::Aptos => 22,
            Self::Arbitrum => 23,
            Self::Optimism => 24,
            Self::Base => 30,
            Self::Linea => 38,
            Self::Berachain => 39,
            Self::Unichain => 44,
            Self::World => 45,
            Self::Hyperevm => 47,
            Self::Monad => 48,
            Self::Sonic => 52,
            Self::Plasma => 58,
            Self::Hypercore => 65000,
        }
    }

    const fn chain(self) -> Chain {
        match self {
            Self::Solana => Chain::Solana,
            Self::Ethereum => Chain::Ethereum,
            Self::Bsc => Chain::SmartChain,
            Self::Polygon => Chain::Polygon,
            Self::Avalanche => Chain::AvalancheC,
            Self::Fantom => Chain::Fantom,
            Self::Ton => Chain::Ton,
            Self::Celo => Chain::Celo,
            Self::Near => Chain::Near,
            Self::Sui => Chain::Sui,
            Self::Aptos => Chain::Aptos,
            Self::Arbitrum => Chain::Arbitrum,
            Self::Optimism => Chain::Optimism,
            Self::Base => Chain::Base,
            Self::Linea => Chain::Linea,
            Self::Berachain => Chain::Berachain,
            Self::Unichain => Chain::Unichain,
            Self::World => Chain::World,
            Self::Hyperevm => Chain::Hyperliquid,
            Self::Monad => Chain::Monad,
            Self::Sonic => Chain::Sonic,
            Self::Plasma => Chain::Plasma,
            Self::Hypercore => Chain::HyperCore,
        }
    }

    pub(in crate::mayan) fn name(self) -> &'static str {
        self.into()
    }
}

pub fn chain_from_id(chain_id: u16) -> Option<Chain> {
    WormholeChain::from_id(chain_id).map(WormholeChain::chain)
}

pub fn chain_for_name(chain: &str) -> Result<Chain, SwapperError> {
    WormholeChain::from_name(chain).map(WormholeChain::chain)
}

pub fn id_for_name(chain: &str) -> Result<u16, SwapperError> {
    WormholeChain::from_name(chain).map(WormholeChain::id)
}

pub fn name_for_chain(chain: Chain) -> Result<&'static str, SwapperError> {
    WormholeChain::from_chain(chain).map(WormholeChain::name).ok_or(SwapperError::NotSupportedChain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_from_id() {
        assert_eq!(chain_from_id(1), Some(Chain::Solana));
        assert_eq!(chain_from_id(2), Some(Chain::Ethereum));
        assert_eq!(chain_from_id(13), Some(Chain::Ton));
        assert_eq!(chain_from_id(21), Some(Chain::Sui));
        assert_eq!(chain_from_id(30), Some(Chain::Base));
        assert_eq!(chain_from_id(9999), None);
    }

    #[test]
    fn test_id_for_name() {
        assert_eq!(id_for_name("solana").unwrap(), 1);
        assert_eq!(id_for_name("ethereum").unwrap(), 2);
        assert_eq!(id_for_name("bsc").unwrap(), 4);
        assert_eq!(id_for_name("sui").unwrap(), 21);
        assert_eq!(id_for_name("base").unwrap(), 30);
        assert_eq!(id_for_name("hyperevm").unwrap(), 47);
        assert_eq!(id_for_name("hypercore").unwrap(), 65000);
        assert_eq!(id_for_name("bitcoin").unwrap_err(), SwapperError::NotSupportedChain);
    }

    #[test]
    fn test_chain_for_name() {
        assert_eq!(chain_for_name("solana").unwrap(), Chain::Solana);
        assert_eq!(chain_for_name("ethereum").unwrap(), Chain::Ethereum);
        assert_eq!(chain_for_name("avalanche").unwrap(), Chain::AvalancheC);
        assert_eq!(chain_for_name("hyperevm").unwrap(), Chain::Hyperliquid);
        assert_eq!(chain_for_name("bitcoin").unwrap_err(), SwapperError::NotSupportedChain);
    }

    #[test]
    fn test_name_for_chain() {
        assert_eq!(name_for_chain(Chain::SmartChain).unwrap(), "bsc");
        assert_eq!(name_for_chain(Chain::AvalancheC).unwrap(), "avalanche");
        assert_eq!(name_for_chain(Chain::Hyperliquid).unwrap(), "hyperevm");
        assert_eq!(name_for_chain(Chain::HyperCore).unwrap(), "hypercore");
        assert_eq!(name_for_chain(Chain::Bitcoin).unwrap_err(), SwapperError::NotSupportedChain);
    }

    #[test]
    fn test_wormhole_chain_names() {
        assert_eq!(WormholeChain::Bsc.name(), "bsc");
        assert_eq!(WormholeChain::Avalanche.name(), "avalanche");
        assert_eq!(WormholeChain::Hyperevm.name(), "hyperevm");
        assert_eq!(WormholeChain::Hypercore.name(), "hypercore");
        assert_eq!(WormholeChain::from_name("bitcoin").unwrap_err(), SwapperError::NotSupportedChain);
    }
}
