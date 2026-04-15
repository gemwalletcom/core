use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::{
    block_explorer::BlockExplorer,
    chain::Chain,
    explorers::{ChainflipScan, MayanScan, NearIntents, RelayScan, RuneScan, SkipExplorer, SocketScan},
};

#[derive(Debug, Copy, Clone, PartialEq, AsRefStr, EnumString, Eq, PartialOrd, Ord, Serialize, Deserialize, EnumIter)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    Aerodrome,
    Panora,
    Thorchain,
    Jupiter,
    Okx,
    Across,
    Oku,
    Wagmi,
    StonfiV2,
    Mayan,
    Chainflip,
    NearIntents,
    CetusAggregator,
    Relay,
    Hyperliquid,
    Orca,
    Squid,
}

impl SwapProvider {
    pub fn id(&self) -> &str {
        self.as_ref()
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn is_cross_chain(&self) -> bool {
        match self {
            Self::Thorchain | Self::Across | Self::Mayan | Self::Chainflip | Self::NearIntents | Self::Relay | Self::Hyperliquid | Self::Squid => true,
            Self::UniswapV3
            | Self::UniswapV4
            | Self::PancakeswapV3
            | Self::Panora
            | Self::Jupiter
            | Self::Okx
            | Self::Oku
            | Self::Wagmi
            | Self::CetusAggregator
            | Self::StonfiV2
            | Self::Aerodrome
            | Self::Orca => false,
        }
    }

    pub fn cross_chain_providers() -> Vec<Self> {
        Self::all().into_iter().filter(Self::is_cross_chain).collect()
    }

    pub fn swap_explorer(&self, chain: Chain) -> Option<Box<dyn BlockExplorer>> {
        match self {
            Self::Mayan => Some(MayanScan::boxed()),
            Self::Thorchain => Some(RuneScan::boxed()),
            Self::Across => Some(SocketScan::boxed()),
            Self::Chainflip => Some(ChainflipScan::boxed()),
            Self::NearIntents => Some(NearIntents::boxed()),
            Self::Relay => Some(RelayScan::boxed()),
            Self::Squid => Some(SkipExplorer::boxed(chain)),
            Self::UniswapV3
            | Self::UniswapV4
            | Self::PancakeswapV3
            | Self::Panora
            | Self::Jupiter
            | Self::Okx
            | Self::Oku
            | Self::Wagmi
            | Self::CetusAggregator
            | Self::StonfiV2
            | Self::Aerodrome
            | Self::Hyperliquid
            | Self::Orca => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 | Self::UniswapV4 => "Uniswap",
            Self::PancakeswapV3 => "PancakeSwap",
            Self::Aerodrome => "Aerodrome",
            Self::Panora => "Panora",
            Self::Thorchain => "THORChain",
            Self::Jupiter => "Jupiter",
            Self::Okx => "OKX (DEX)",
            Self::Across => "Across",
            Self::Oku => "Oku",
            Self::Wagmi => "Wagmi",
            Self::CetusAggregator => "Cetus",
            Self::StonfiV2 => "STON.fi",
            Self::Mayan => "Mayan",
            Self::Chainflip => "Chainflip",
            Self::NearIntents => "NEAR Intents",
            Self::Relay => "Relay",
            Self::Hyperliquid => "Hyperliquid",
            Self::Orca => "Orca",
            Self::Squid => "Squid",
        }
    }

    pub fn protocol_name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::UniswapV4 => "Uniswap v4",
            Self::PancakeswapV3 => "PancakeSwap v3",
            Self::Panora => "Panora",
            Self::Across => "Across v3",
            Self::Oku => "Oku",
            Self::StonfiV2 => "STON.fi v2",
            Self::Thorchain
            | Self::Jupiter
            | Self::Okx
            | Self::Wagmi
            | Self::Mayan
            | Self::Chainflip
            | Self::NearIntents
            | Self::CetusAggregator
            | Self::Aerodrome
            | Self::Relay
            | Self::Hyperliquid
            | Self::Orca
            | Self::Squid => self.name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cross_chain() {
        assert!(SwapProvider::Thorchain.is_cross_chain());
        assert!(SwapProvider::Across.is_cross_chain());
        assert!(SwapProvider::Mayan.is_cross_chain());
        assert!(SwapProvider::NearIntents.is_cross_chain());
        assert!(SwapProvider::Relay.is_cross_chain());
        assert!(!SwapProvider::UniswapV3.is_cross_chain());
        assert!(!SwapProvider::Jupiter.is_cross_chain());
    }
}
