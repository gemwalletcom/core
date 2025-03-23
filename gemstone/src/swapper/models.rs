use super::permit2_data::Permit2Data;
use crate::config::swap_config::{SwapReferralFees, DEFAULT_SLIPPAGE_BPS};
use primitives::{AssetId, Chain};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapMode {
    ExactIn,
    ExactOut,
}

#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub struct SwapProviderConfig(SwapProviderType);

impl SwapProviderConfig {
    pub fn id(&self) -> SwapProvider {
        self.0.id.clone()
    }
}

#[uniffi::export]
impl SwapProviderConfig {
    #[uniffi::constructor]
    pub fn new(id: SwapProvider) -> Self {
        Self(SwapProviderType::new(id))
    }
    pub fn inner(&self) -> SwapProviderType {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SwapProviderType {
    pub id: SwapProvider,
    pub mode: SwapProviderMode,
    pub name: String,
    pub protocol: String,
}

impl SwapProviderType {
    pub fn new(id: SwapProvider) -> Self {
        Self {
            id: id.clone(),
            mode: id.mode(),
            name: id.name().to_string(),
            protocol: id.protocol_name().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SwapProviderMode {
    OnChain,
    CrossChain,
    Bridge,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, uniffi::Enum)]
pub enum SwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeSwapV3,
    PancakeSwapAptosV2,
    Thorchain,
    Orca,
    Jupiter,
    Across,
    OkuTrade,
    Wagmi,
    Cetus,
    MayanSwift,
}

impl SwapProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 | Self::UniswapV4 => "Uniswap",
            Self::PancakeSwapV3 | Self::PancakeSwapAptosV2 => "PancakeSwap",
            Self::Thorchain => "THORChain",
            Self::Orca => "Orca",
            Self::Jupiter => "Jupiter",
            Self::Across => "Across",
            Self::OkuTrade => "Oku",
            Self::Wagmi => "Wagmi",
            Self::Cetus => "Cetus",
            Self::MayanSwift => "Mayan",
        }
    }

    pub fn protocol_name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::UniswapV4 => "Uniswap v4",
            Self::PancakeSwapV3 => "PancakeSwap v3",
            Self::PancakeSwapAptosV2 => "PancakeSwap v2",
            Self::Thorchain => "THORChain",
            Self::Orca => "Orca Whirlpool",
            Self::Jupiter => "Jupiter",
            Self::Across => "Across v3",
            Self::OkuTrade => "Oku Trade",
            Self::Wagmi => "Wagmi",
            Self::Cetus => "Cetus",
            Self::MayanSwift => "Mayan Swift",
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
            | Self::OkuTrade
            | Self::Wagmi
            | Self::Cetus => SwapProviderMode::OnChain,
            Self::Thorchain | Self::MayanSwift => SwapProviderMode::CrossChain,
            Self::Across => SwapProviderMode::Bridge,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapQuoteRequest {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: String,
    pub mode: GemSwapMode,
    pub options: GemSwapOptions,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSlippage {
    pub bps: u32,
    pub mode: SlippageMode,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SlippageMode {
    Auto,
    Exact,
}

impl From<u32> for GemSlippage {
    fn from(value: u32) -> Self {
        GemSlippage {
            bps: value,
            mode: SlippageMode::Exact,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapOptions {
    pub slippage: GemSlippage,
    pub fee: Option<SwapReferralFees>,
    pub preferred_providers: Vec<SwapProvider>,
}

impl Default for GemSwapOptions {
    fn default() -> Self {
        Self {
            slippage: DEFAULT_SLIPPAGE_BPS.into(),
            fee: None,
            preferred_providers: vec![],
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapQuote {
    pub from_value: String,
    // Original quote amount
    pub to_value: String,
    // Minimum amount (slippage, fee applied)
    pub to_min_value: String,
    pub data: SwapProviderData,
    pub request: SwapQuoteRequest,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum ApprovalType {
    Approve(ApprovalData),
    Permit2(Permit2ApprovalData),
    None,
}

impl ApprovalType {
    pub fn approval_data(&self) -> Option<ApprovalData> {
        match self {
            Self::Approve(data) => Some(data.clone()),
            _ => None,
        }
    }
    pub fn permit2_data(&self) -> Option<Permit2ApprovalData> {
        match self {
            Self::Permit2(data) => Some(data.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Permit2ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub permit2_contract: String,
    pub permit2_nonce: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapProviderData {
    pub provider: SwapProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<SwapRoute>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapRoute {
    pub input: AssetId,
    pub output: AssetId,
    pub route_data: String,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FetchQuoteData {
    Permit2(Permit2Data),
    EstimateGas,
    None,
}

impl FetchQuoteData {
    pub fn permit2_data(&self) -> Option<Permit2Data> {
        match self {
            Self::Permit2(data) => Some(data.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum, PartialEq)]
pub enum SwapChainAsset {
    All(Chain),
    Assets(Chain, Vec<AssetId>),
}

impl SwapChainAsset {
    pub fn get_chain(&self) -> Chain {
        match self {
            Self::All(chain) => *chain,
            Self::Assets(chain, _) => *chain,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record, PartialEq)]
pub struct SwapAssetList {
    pub chains: Vec<Chain>,
    pub asset_ids: Vec<AssetId>,
}
