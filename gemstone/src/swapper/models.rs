use super::permit2_data::Permit2Data;
use crate::{
    config::swap_config::{SwapReferralFees, DEFAULT_SLIPPAGE_BPS},
    network::{jsonrpc::JsonRpcError, AlienError},
};
use gem_evm::address::AddressError;
use primitives::{AssetId, Chain};
use std::fmt::Debug;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SwapperError {
    #[error("Not supported chain")]
    NotSupportedChain,
    #[error("Not supported asset")]
    NotSupportedAsset,
    #[error("Not supported pair")]
    NotSupportedPair,
    #[error("No available provider")]
    NoAvailableProvider,
    #[error("Invalid address {address}")]
    InvalidAddress { address: String },
    #[error("Invalid input amount")]
    InvalidAmount,
    #[error("Invalid route")]
    InvalidRoute,
    #[error("RPC error: {msg}")]
    NetworkError { msg: String },
    #[error("ABI error: {msg}")]
    ABIError { msg: String },
    #[error("Compute quote error: {msg}")]
    ComputeQuoteError { msg: String },

    #[error("No quote available")]
    NoQuoteAvailable,
    #[error("Not implemented")]
    NotImplemented,
}

impl From<AlienError> for SwapperError {
    fn from(err: AlienError) -> Self {
        Self::NetworkError { msg: err.to_string() }
    }
}

impl From<JsonRpcError> for SwapperError {
    fn from(err: JsonRpcError) -> Self {
        Self::NetworkError { msg: err.to_string() }
    }
}

impl From<AddressError> for SwapperError {
    fn from(err: AddressError) -> Self {
        Self::InvalidAddress { address: err.to_string() }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapMode {
    ExactIn,
    ExactOut,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, uniffi::Enum)]
pub enum SwapProvider {
    UniswapV3,
    PancakeSwapV3,
    PancakeSwapAptosV2,
    Thorchain,
    Orca,
    Jupiter,
    Across,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SwapProviderType {
    OnChain,
    CrossChain,
    Bridge,
}

impl SwapProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::PancakeSwapV3 => "PancakeSwap v3",
            Self::PancakeSwapAptosV2 => "PancakeSwap v2",
            Self::Thorchain => "THORChain",
            Self::Orca => "Orca Whirlpool",
            Self::Jupiter => "Jupiter",
            Self::Across => "Across v3",
        }
    }

    pub fn provider_type(&self) -> SwapProviderType {
        match self {
            Self::UniswapV3 => SwapProviderType::OnChain,
            Self::PancakeSwapV3 => SwapProviderType::OnChain,
            Self::PancakeSwapAptosV2 => SwapProviderType::OnChain,
            Self::Thorchain => SwapProviderType::CrossChain,
            Self::Orca => SwapProviderType::OnChain,
            Self::Jupiter => SwapProviderType::OnChain,
            Self::Across => SwapProviderType::Bridge,
        }
    }
}

#[uniffi::export]
fn swap_provider_name_to_string(provider: SwapProvider) -> String {
    provider.name().to_string()
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSlippage {
    pub bps: u32,
    pub mode: SlippageMode,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum SlippageMode {
    Auto,
    Exact,
    Max,
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
    pub to_value: String,
    pub data: SwapProviderData,
    pub approval: ApprovalType,
    pub request: SwapQuoteRequest,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum ApprovalType {
    Approve(ApprovalData),
    Permit2(Permit2ApprovalData),
    None,
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
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapProviderData {
    pub provider: SwapProvider,
    pub slippage_bps: u32,
    pub routes: Vec<SwapRoute>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapRoute {
    pub input: AssetId,
    pub output: AssetId,
    pub route_data: String,
    pub gas_estimate: Option<String>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FetchQuoteData {
    Permit2(Permit2Data),
    EstimateGas,
    None,
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
