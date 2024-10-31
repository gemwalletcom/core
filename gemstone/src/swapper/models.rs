use alloy_core::primitives::{Address, Bytes, U256};
use alloy_primitives::aliases::{U160, U48};
use gem_evm::{permit2::IAllowanceTransfer, uniswap::command::Permit2Permit};
use primitives::{AssetId, ChainType};
use std::{fmt::Debug, str::FromStr};

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
pub struct SwapQuoteRequest {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub wallet_address: String,
    pub destination_address: String,
    pub amount: String,
    pub mode: GemSwapMode,
    pub options: Option<GemSwapOptions>,
}

#[derive(Debug, Default, Clone, uniffi::Record)]
pub struct GemSwapFee {
    pub bps: u32,
    pub address: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapOptions {
    pub slippage_bps: u32,
    pub fee: Option<GemSwapFee>,
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
pub struct GemSwapQuote {
    pub chain_type: ChainType,
    pub from_value: String,
    pub to_value: String,
    pub provider: GemProviderData,
    pub approval: ApprovalType,
    pub request: SwapQuoteRequest,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum ApprovalType {
    Approve(ApprovalData),
    Permit2(ApprovalData),
    None,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
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
    pub routes: Vec<SwapRoute>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapRoute {
    pub route_type: String,
    pub input: String,
    pub output: String,
    pub fee_tier: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Permit2Detail {
    pub token: String,
    pub amount: String,
    pub expiration: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PermitSingle {
    pub details: Permit2Detail,
    pub spender: String,
    pub sig_deadline: u64,
}

impl From<PermitSingle> for IAllowanceTransfer::PermitSingle {
    fn from(val: PermitSingle) -> Self {
        IAllowanceTransfer::PermitSingle {
            details: IAllowanceTransfer::PermitDetails {
                token: Address::parse_checksummed(val.details.token, None).unwrap(),
                amount: U160::from_str(&val.details.amount).unwrap(),
                expiration: U48::from(val.details.expiration),
                nonce: U48::from(val.details.nonce),
            },
            spender: Address::parse_checksummed(val.spender, None).unwrap(),
            sigDeadline: U256::from(val.sig_deadline),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPermit2Data {
    pub permit_single: PermitSingle,
    pub signature: Vec<u8>,
}

impl From<GemPermit2Data> for Permit2Permit {
    fn from(val: GemPermit2Data) -> Self {
        Permit2Permit {
            permit_single: val.permit_single.into(),
            signature: Bytes::from(val.signature),
        }
    }
}
