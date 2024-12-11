use super::permit2_data::Permit2Data;
use crate::config::swap_config::SwapReferralFees;
use crate::network::{jsonrpc::JsonRpcError, AlienError};
use primitives::AssetId;
use std::fmt::Debug;

static DEFAULT_SLIPPAGE_BPS: u32 = 300;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SwapperError {
    #[error("Not supported chain")]
    NotSupportedChain,
    #[error("Not supported asset")]
    NotSupportedAsset,
    #[error("Not supported pair")]
    NotSupportedPair,
    #[error("Invalid address {address}")]
    InvalidAddress { address: String },
    #[error("Invalid amount")]
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapMode {
    ExactIn,
    ExactOut,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SwapProvider {
    UniswapV3,
    PancakeSwapV3,
    Thorchain,
    Orca,
    Jupiter,
    Across,
}
impl SwapProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::PancakeSwapV3 => "PancakeSwap v3",
            Self::Thorchain => "THORChain",
            Self::Orca => "Orca Whirlpool",
            Self::Jupiter => "Jupiter",
            Self::Across => "Across v3",
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
    pub options: Option<GemSwapOptions>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapOptions {
    pub slippage_bps: u32,
    pub fee: Option<SwapReferralFees>,
    pub preferred_providers: Vec<SwapProvider>,
}

impl Default for GemSwapOptions {
    fn default() -> Self {
        Self {
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
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
    pub suggested_slippage_bps: Option<u32>,
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
    None,
}
