use super::permit2_data::Permit2Data;
use crate::{
    SwapperMode, SwapperProvider, SwapperQuoteAsset, SwapperSlippage,
    config::{DEFAULT_SLIPPAGE_BPS, ReferralFees},
};
pub use primitives::swap::SwapResult;
use primitives::{
    AssetId, Chain,
    swap::{ApprovalData, SwapProviderMode},
};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderType {
    pub id: SwapperProvider,
    pub name: String,
    pub protocol: String,
    pub protocol_id: String,
    pub mode: SwapProviderMode,
}

impl ProviderType {
    pub fn new(id: SwapperProvider) -> Self {
        Self {
            id,
            name: id.name().to_string(),
            protocol: id.protocol_name().to_string(),
            protocol_id: id.id().to_string(),
            mode: ProviderType::mode(id),
        }
    }

    pub fn mode(id: SwapperProvider) -> SwapProviderMode {
        match id {
            SwapperProvider::UniswapV3
            | SwapperProvider::UniswapV4
            | SwapperProvider::PancakeswapV3
            | SwapperProvider::PancakeswapAptosV2
            | SwapperProvider::Jupiter
            | SwapperProvider::Oku
            | SwapperProvider::Wagmi
            | SwapperProvider::Hyperswap
            | SwapperProvider::Cetus
            | SwapperProvider::CetusAggregator
            | SwapperProvider::StonfiV2
            | SwapperProvider::Reservoir
            | SwapperProvider::Aerodrome
            | SwapperProvider::Orca => SwapProviderMode::OnChain,
            SwapperProvider::Mayan | SwapperProvider::Chainflip | SwapperProvider::NearIntents => SwapProviderMode::CrossChain,
            SwapperProvider::Thorchain => SwapProviderMode::OmniChain(vec![Chain::Thorchain, Chain::Tron]),
            SwapperProvider::Relay => SwapProviderMode::OmniChain(vec![Chain::Hyperliquid, Chain::Manta, Chain::Berachain]),
            SwapperProvider::Across | SwapperProvider::Hyperliquid => SwapProviderMode::Bridge,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRequest {
    pub from_asset: SwapperQuoteAsset,
    pub to_asset: SwapperQuoteAsset,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: String,
    pub mode: SwapperMode,
    pub options: Options,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub slippage: SwapperSlippage,
    pub fee: Option<ReferralFees>,
    pub preferred_providers: Vec<SwapperProvider>,
    pub use_max_amount: bool,
}

impl Options {
    pub fn new_with_slippage(slippage: SwapperSlippage) -> Self {
        Self {
            slippage,
            ..Default::default()
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            slippage: DEFAULT_SLIPPAGE_BPS.into(),
            fee: None,
            preferred_providers: vec![],
            use_max_amount: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub from_value: String,
    pub to_value: String,
    pub data: ProviderData,
    pub request: QuoteRequest,
    pub eta_in_seconds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Permit2ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub permit2_contract: String,
    pub permit2_nonce: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderData {
    pub provider: ProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub input: AssetId,
    pub output: AssetId,
    pub route_data: String,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum SwapperChainAsset {
    All(Chain),
    Assets(Chain, Vec<AssetId>),
}

impl SwapperChainAsset {
    pub fn get_chain(&self) -> Chain {
        match self {
            Self::All(chain) => *chain,
            Self::Assets(chain, _) => *chain,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetList {
    pub chains: Vec<Chain>,
    pub asset_ids: Vec<AssetId>,
}
