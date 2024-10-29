use primitives::{AssetId, ChainType};
use std::fmt::Debug;

static DEFAULT_SLIPPAGE_BPS: u32 = 300;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GemSwapperError {
    #[error("Not supported chain")]
    NotSupportedChain,
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapRequest {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub wallet_address: String,
    pub destination_address: String,
    pub amount: String,
    pub mode: GemSwapMode,
    pub options: Option<GemSwapOptions>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapOptions {
    pub slippage_bps: u32,
    pub fee_bps: u32,
    pub fee_address: String,
    pub preferred_providers: Vec<String>,
}

impl Default for GemSwapOptions {
    fn default() -> Self {
        Self {
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            preferred_providers: vec![],
            fee_bps: 0,
            fee_address: String::from(""),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapQuote {
    pub chain_type: ChainType,
    pub from_amount: String,
    pub to_amount: String,
    pub provider: GemProviderData,
    pub approval: Option<GemApprovalData>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemApprovalData {
    pub spender: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemProviderData {
    pub name: String,
    pub route: GemSwapRoute,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapRoute {
    pub route_type: String,
    pub input: String,
    pub output: String,
    pub fee: String,
}
