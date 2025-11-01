use crate::config::swap_config::SwapReferralFees;
use primitives::{AssetId, Chain};
use std::str::FromStr;
pub use swapper::{
    AssetList as SwapperAssetList, FetchQuoteData, Options as SwapperOptions, ProviderData as SwapperProviderData, ProviderType as SwapperProviderType,
    Quote as SwapperQuote, QuoteRequest as SwapperQuoteRequest, Route as SwapperRoute, SwapResult as SwapperSwapResult, SwapperMode, SwapperProvider,
    SwapperProviderMode, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode, SwapperSwapStatus, permit2_data::Permit2Data,
};

pub use crate::models::swap::GemSwapQuoteData;

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
    pub mode: SwapperProviderMode,
}

#[uniffi::remote(Record)]
pub struct SwapperOptions {
    pub slippage: SwapperSlippage,
    pub fee: Option<SwapReferralFees>,
    pub preferred_providers: Vec<SwapperProvider>,
    pub use_max_amount: bool,
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
    Hyperswap,
    Cetus,
    StonfiV2,
    Mayan,
    Reservoir,
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

#[uniffi::export]
fn swapper_provider_from_str(s: &str) -> Option<SwapperProvider> {
    SwapperProvider::from_str(s).ok()
}

#[uniffi::export]
fn swapper_provider_to_str(provider: SwapperProvider) -> String {
    provider.as_ref().to_string()
}
