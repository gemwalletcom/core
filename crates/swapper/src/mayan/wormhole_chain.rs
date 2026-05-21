use crate::SwapperError;
use primitives::Chain;
use std::str::FromStr;
use strum::{EnumString, IntoStaticStr};

macro_rules! wormhole_chains {
    ($($variant:ident => { id: $id:expr, chain: $chain:path, name: $name:literal }),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
        pub(in crate::mayan) enum WormholeChain {
            $(
                #[strum(serialize = $name)]
                $variant,
            )+
        }

        impl WormholeChain {
            pub(in crate::mayan) fn from_name(name: &str) -> Result<Self, SwapperError> {
                Self::from_str(name).map_err(|_| SwapperError::NotSupportedChain)
            }

            fn from_id(chain_id: u16) -> Option<Self> {
                match chain_id {
                    $($id => Some(Self::$variant),)+
                    _ => None,
                }
            }

            fn from_chain(chain: Chain) -> Option<Self> {
                match chain {
                    $($chain => Some(Self::$variant),)+
                    _ => None,
                }
            }

            pub(in crate::mayan) const fn id(self) -> u16 {
                match self {
                    $(Self::$variant => $id,)+
                }
            }

            const fn chain(self) -> Chain {
                match self {
                    $(Self::$variant => $chain,)+
                }
            }

            pub(in crate::mayan) fn name(self) -> &'static str {
                self.into()
            }
        }
    };
}

wormhole_chains! {
    Solana => { id: 1, chain: Chain::Solana, name: "solana" },
    Ethereum => { id: 2, chain: Chain::Ethereum, name: "ethereum" },
    Bsc => { id: 4, chain: Chain::SmartChain, name: "bsc" },
    Polygon => { id: 5, chain: Chain::Polygon, name: "polygon" },
    Avalanche => { id: 6, chain: Chain::AvalancheC, name: "avalanche" },
    Fantom => { id: 10, chain: Chain::Fantom, name: "fantom" },
    Ton => { id: 13, chain: Chain::Ton, name: "ton" },
    Celo => { id: 14, chain: Chain::Celo, name: "celo" },
    Near => { id: 15, chain: Chain::Near, name: "near" },
    Sui => { id: 21, chain: Chain::Sui, name: "sui" },
    Aptos => { id: 22, chain: Chain::Aptos, name: "aptos" },
    Arbitrum => { id: 23, chain: Chain::Arbitrum, name: "arbitrum" },
    Optimism => { id: 24, chain: Chain::Optimism, name: "optimism" },
    Base => { id: 30, chain: Chain::Base, name: "base" },
    Linea => { id: 38, chain: Chain::Linea, name: "linea" },
    Berachain => { id: 39, chain: Chain::Berachain, name: "berachain" },
    Unichain => { id: 44, chain: Chain::Unichain, name: "unichain" },
    World => { id: 45, chain: Chain::World, name: "world" },
    Hyperevm => { id: 47, chain: Chain::Hyperliquid, name: "hyperevm" },
    Monad => { id: 48, chain: Chain::Monad, name: "monad" },
    Sonic => { id: 52, chain: Chain::Sonic, name: "sonic" },
    Plasma => { id: 58, chain: Chain::Plasma, name: "plasma" },
    Hypercore => { id: 65000, chain: Chain::HyperCore, name: "hypercore" },
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
