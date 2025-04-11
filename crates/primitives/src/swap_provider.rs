use strum::EnumString;
use strum_macros::AsRefStr;
use typeshare::typeshare;

#[derive(Debug, Copy, Clone, PartialEq, AsRefStr, EnumString, Eq, PartialOrd, Ord)]
#[typeshare(swift = "Equatable, Sendable")]
#[strum(serialize_all = "lowercase")]
pub enum SwapProviderMode {
    OnChain,
    CrossChain,
    Bridge,
}

#[derive(Debug, Copy, Clone, PartialEq, AsRefStr, EnumString, Eq, PartialOrd, Ord)]
#[typeshare(swift = "Equatable, Sendable")]
#[strum(serialize_all = "lowercase")]
pub enum SwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeSwapV3,
    PancakeSwapAptosV2,
    Thorchain,
    Orca,
    Jupiter,
    Across,
    Oku,
    Wagmi,
    Cetus,
    StonFiV2,
    Mayan,
    Reservoir,
}

impl SwapProvider {
    pub fn id(&self) -> &str {
        self.as_ref()
    }

    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 | Self::UniswapV4 => "Uniswap",
            Self::PancakeSwapV3 | Self::PancakeSwapAptosV2 => "PancakeSwap",
            Self::Thorchain => "THORChain",
            Self::Orca => "Orca",
            Self::Jupiter => "Jupiter",
            Self::Across => "Across",
            Self::Oku => "Oku",
            Self::Wagmi => "Wagmi",
            Self::Cetus => "Cetus",
            Self::StonFiV2 => "STON.fi",
            Self::Mayan => "Mayan",
            Self::Reservoir => "Reservoir",
        }
    }

    pub fn protocol_name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::UniswapV4 => "Uniswap v4",
            Self::PancakeSwapV3 => "PancakeSwap v3",
            Self::PancakeSwapAptosV2 => "PancakeSwap v2",
            Self::Orca => "Orca Whirlpool",
            Self::Across => "Across v3",
            Self::Oku => "Oku",
            Self::StonFiV2 => "STON.fi v2",
            Self::Thorchain | Self::Jupiter | Self::Wagmi | Self::Cetus | Self::Mayan | Self::Reservoir => self.name(),
        }
    }

    pub fn mode(&self) -> SwapProviderMode {
        match self {
            Self::UniswapV3
            | Self::UniswapV4
            | Self::PancakeSwapV3
            | Self::PancakeSwapAptosV2
            | Self::Orca
            | Self::Jupiter
            | Self::Oku
            | Self::Wagmi
            | Self::Cetus
            | Self::StonFiV2
            | Self::Reservoir => SwapProviderMode::OnChain,
            Self::Thorchain | Self::Mayan => SwapProviderMode::CrossChain,
            Self::Across => SwapProviderMode::Bridge,
        }
    }
}
