use serde::{Deserialize, Serialize};
use strum::EnumString;
use strum_macros::AsRefStr;
use typeshare::typeshare;

#[derive(Debug, Copy, Clone, PartialEq, AsRefStr, EnumString, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    PancakeswapAptosV2,
    Thorchain,
    Orca,
    Jupiter,
    Across,
    Oku,
    Wagmi,
    Cetus,
    StonfiV2,
    Mayan,
    Reservoir,
    Symbiosis,
    Chainflip,
}

impl SwapProvider {
    pub fn id(&self) -> &str {
        self.as_ref()
    }

    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 | Self::UniswapV4 => "Uniswap",
            Self::PancakeswapV3 | Self::PancakeswapAptosV2 => "PancakeSwap",
            Self::Thorchain => "THORChain",
            Self::Orca => "Orca",
            Self::Jupiter => "Jupiter",
            Self::Across => "Across",
            Self::Oku => "Oku",
            Self::Wagmi => "Wagmi",
            Self::Cetus => "Cetus",
            Self::StonfiV2 => "STON.fi",
            Self::Mayan => "Mayan",
            Self::Reservoir => "Reservoir",
            Self::Symbiosis => "Symbiosis",
            Self::Chainflip => "Chainflip",
        }
    }

    pub fn protocol_name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::UniswapV4 => "Uniswap v4",
            Self::PancakeswapV3 => "PancakeSwap v3",
            Self::PancakeswapAptosV2 => "PancakeSwap v2",
            Self::Orca => "Orca Whirlpool",
            Self::Across => "Across v3",
            Self::Oku => "Oku",
            Self::StonfiV2 => "STON.fi v2",
            Self::Thorchain | Self::Jupiter | Self::Wagmi | Self::Cetus | Self::Mayan | Self::Reservoir | Self::Symbiosis | Self::Chainflip => self.name(),
        }
    }
}
