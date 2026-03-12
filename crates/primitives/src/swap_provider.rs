use crate::PrioritizedProvider;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

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
            Self::Thorchain | Self::Across | Self::Mayan | Self::Chainflip | Self::NearIntents | Self::Relay | Self::Hyperliquid => true,
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
            | Self::Orca => self.name(),
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            Self::UniswapV3 | Self::UniswapV4 | Self::PancakeswapV3 | Self::Aerodrome | Self::Oku | Self::Wagmi | Self::StonfiV2 | Self::Orca | Self::Hyperliquid => 1,
            Self::Thorchain | Self::Across | Self::Mayan | Self::Chainflip | Self::NearIntents | Self::Relay => 2,
            Self::Jupiter | Self::Okx | Self::CetusAggregator | Self::Panora => 3,
        }
    }

    pub fn threshold_bps(&self) -> i32 {
        match self.priority() {
            1 => 0,
            _ => 200,
        }
    }
}

impl PrioritizedProvider for SwapProvider {
    fn provider_id(&self) -> &str {
        self.id()
    }

    fn priority(&self) -> i32 {
        SwapProvider::priority(self)
    }

    fn threshold_bps(&self) -> i32 {
        SwapProvider::threshold_bps(self)
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

    #[test]
    fn test_priority() {
        assert_eq!(SwapProvider::UniswapV3.priority(), 1);
        assert_eq!(SwapProvider::Jupiter.priority(), 3);
        assert_eq!(SwapProvider::Thorchain.priority(), 2);
        assert_eq!(SwapProvider::Mayan.priority(), 2);
        assert_eq!(SwapProvider::Okx.priority(), 3);
    }

    #[test]
    fn test_threshold_bps() {
        assert_eq!(SwapProvider::UniswapV3.threshold_bps(), 0);
        assert_eq!(SwapProvider::Thorchain.threshold_bps(), 200);
        assert_eq!(SwapProvider::Okx.threshold_bps(), 200);
    }
}
