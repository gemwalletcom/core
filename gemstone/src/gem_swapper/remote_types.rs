use crate::config::swap_config::SwapReferralFees;
use primitives::{AssetId, Chain, swap::ApprovalData as GemApprovalData};
use std::str::FromStr;
pub use swapper::{
    AssetList as SwapperAssetList, FetchQuoteData, Options as SwapperOptions, ProviderData as SwapperProviderData, ProviderType as SwapperProviderType,
    Quote as SwapperQuote, QuoteRequest as SwapperQuoteRequest, Route as SwapperRoute, SwapResult as SwapperSwapResult, SwapperMode, SwapperProvider,
    SwapperProviderMode, SwapperQuoteAsset, SwapperQuoteData, SwapperSlippage, SwapperSlippageMode, SwapperSwapStatus, permit2_data::Permit2Data,
};

#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub struct SwapProviderConfig(SwapperProviderType);

#[uniffi::export]
impl SwapProviderConfig {
    #[uniffi::constructor]
    pub fn new(id: SwapperProvider) -> Self {
        Self(SwapperProviderType::new(id))
    }
    #[uniffi::constructor]
    pub fn from_string(id: String) -> Self {
        let id = SwapperProvider::from_str(&id).unwrap();
        Self(SwapperProviderType::new(id))
    }
    pub fn inner(&self) -> SwapperProviderType {
        self.0.clone()
    }
}

#[uniffi::remote(Enum)]
pub enum FetchQuoteData {
    Permit2(Permit2Data),
    EstimateGas,
    None,
}

#[uniffi::remote(Record)]
pub struct SwapperSwapResult {
    pub status: SwapperSwapStatus,
    pub from_chain: Chain,
    pub from_tx_hash: String,
    pub to_chain: Option<Chain>,
    pub to_tx_hash: Option<String>,
}

#[uniffi::remote(Record)]
pub struct SwapperAssetList {
    pub chains: Vec<Chain>,
    pub asset_ids: Vec<AssetId>,
}

#[uniffi::remote(Record)]
pub struct SwapperProviderType {
    pub id: SwapperProvider,
    pub name: String,
    pub protocol: String,
    pub protocol_id: String,
}

#[uniffi::remote(Record)]
pub struct SwapperOptions {
    pub slippage: SwapperSlippage,
    pub fee: Option<SwapReferralFees>,
    pub preferred_providers: Vec<SwapperProvider>,
}

#[uniffi::remote(Record)]
pub struct SwapperQuoteRequest {
    pub from_asset: SwapperQuoteAsset,
    pub to_asset: SwapperQuoteAsset,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: String,
    pub mode: SwapperMode,
    pub options: SwapperOptions,
}

#[uniffi::remote(Record)]
pub struct SwapperRoute {
    pub input: AssetId,
    pub output: AssetId,
    pub route_data: String,
    pub gas_limit: Option<String>,
}

#[uniffi::remote(Record)]
pub struct SwapperProviderData {
    pub provider: SwapperProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<SwapperRoute>,
}

#[uniffi::remote(Record)]
pub struct SwapperQuote {
    pub from_value: String,
    pub to_value: String,
    pub data: SwapperProviderData,
    pub request: SwapperQuoteRequest,
    pub eta_in_seconds: Option<u32>,
}

#[uniffi::remote(Record)]
pub struct SwapperQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub approval: Option<GemApprovalData>,
    pub gas_limit: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum SwapperProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    Aerodrome,
    PancakeswapAptosV2,
    Thorchain,
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
    NearIntents,
    CetusAggregator,
    Relay,
    Hyperliquid,
    Orca,
}

#[uniffi::remote(Enum)]
pub enum SwapperProviderMode {
    OnChain,
    CrossChain,
    Bridge,
    OmniChain(Vec<Chain>),
}

#[uniffi::remote(Enum)]
pub enum SwapperMode {
    ExactIn,
    ExactOut,
}

#[uniffi::remote(Record)]
pub struct SwapperSlippage {
    pub bps: u32,
    pub mode: SwapperSlippageMode,
}

#[uniffi::remote(Enum)]
pub enum SwapperSlippageMode {
    Auto,
    Exact,
}

#[uniffi::remote(Record)]
pub struct SwapperQuoteAsset {
    pub id: String,
    pub symbol: String,
    pub decimals: u32,
}

#[uniffi::remote(Enum)]
pub enum SwapperSwapStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}
