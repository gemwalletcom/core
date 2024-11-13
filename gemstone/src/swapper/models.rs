use primitives::{AssetId, ChainType};
use std::fmt::Debug;

use crate::config::swap_config::SwapReferralFees;

use super::permit2_data::Permit2Data;

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
    #[error("RPC error: {msg}")]
    NetworkError { msg: String },
    #[error("ABI error: {msg}")]
    ABIError { msg: String },
    #[error("No quote available")]
    NoQuoteAvailable,
    #[error("Not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapMode {
    ExactIn,
    ExactOut,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SwapProvider {
    UniswapV3,
    Thorchain,
}
impl SwapProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::Thorchain => "THORChain",
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
    pub options: Option<GemSwapOptions>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapOptions {
    pub slippage_bps: u32,
    pub fee: Option<SwapReferralFees>,
    pub preferred_providers: Vec<String>,
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
    pub chain_type: ChainType,
    pub from_value: String,
    pub to_value: String,
    pub provider: SwapProviderData,
    pub approval: ApprovalType,
    pub request: SwapQuoteRequest,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum ApprovalType {
    Approve(ApprovalData),
    Permit2(Permit2ApprovalData),
    None,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Permit2ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
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
    pub routes: Vec<SwapRoute>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapRoute {
    pub route_type: String,
    pub input: String,
    pub output: String,
    pub fee_tier: String,
    pub gas_estimate: Option<String>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FetchQuoteData {
    Permit2(Permit2Data),
    None,
}
