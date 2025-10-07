use super::permit2_data::Permit2Data;
use crate::{
    SwapperMode, SwapperProvider, SwapperProviderMode, SwapperQuoteAsset, SwapperSlippage, SwapperSwapStatus,
    config::{DEFAULT_SLIPPAGE_BPS, ReferralFees},
};
use primitives::{AssetId, Chain, swap::ApprovalData};
use std::{fmt::Debug, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct SwapProviderConfig(SwapperProviderType);

impl SwapProviderConfig {
    pub fn id(&self) -> SwapperProvider {
        self.0.id
    }
}

impl SwapProviderConfig {
    pub fn new(id: SwapperProvider) -> Self {
        Self(SwapperProviderType::new(id))
    }
    pub fn from_string(id: String) -> Self {
        let id = SwapperProvider::from_str(&id).unwrap();
        Self(SwapperProviderType::new(id))
    }
    pub fn inner(&self) -> SwapperProviderType {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapperProviderType {
    pub id: SwapperProvider,
    pub name: String,
    pub protocol: String,
    pub protocol_id: String,
}

impl SwapperProviderType {
    pub fn new(id: SwapperProvider) -> Self {
        Self {
            id,
            name: id.name().to_string(),
            protocol: id.protocol_name().to_string(),
            protocol_id: id.id().to_string(),
        }
    }

    pub fn mode(&self) -> SwapperProviderMode {
        match self.id {
            SwapperProvider::UniswapV3
            | SwapperProvider::UniswapV4
            | SwapperProvider::PancakeswapV3
            | SwapperProvider::PancakeswapAptosV2
            | SwapperProvider::Jupiter
            | SwapperProvider::Oku
            | SwapperProvider::Wagmi
            | SwapperProvider::Cetus
            | SwapperProvider::CetusAggregator
            | SwapperProvider::StonfiV2
            | SwapperProvider::Reservoir
            | SwapperProvider::Symbiosis
            | SwapperProvider::Aerodrome => SwapperProviderMode::OnChain,
            SwapperProvider::Mayan | SwapperProvider::Chainflip => SwapperProviderMode::CrossChain,
            SwapperProvider::Thorchain => SwapperProviderMode::OmniChain(vec![Chain::Thorchain, Chain::Tron]),
            SwapperProvider::Relay => SwapperProviderMode::OmniChain(vec![Chain::Hyperliquid, Chain::Manta, Chain::Berachain]),
            SwapperProvider::Across | SwapperProvider::Hyperliquid => SwapperProviderMode::Bridge,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapperQuoteRequest {
    pub from_asset: SwapperQuoteAsset,
    pub to_asset: SwapperQuoteAsset,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: String,
    pub mode: SwapperMode,
    pub options: SwapperOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapperOptions {
    pub slippage: SwapperSlippage,
    pub fee: Option<ReferralFees>,
    pub preferred_providers: Vec<SwapperProvider>,
}

impl Default for SwapperOptions {
    fn default() -> Self {
        Self {
            slippage: DEFAULT_SLIPPAGE_BPS.into(),
            fee: None,
            preferred_providers: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapperQuote {
    pub from_value: String,
    pub to_value: String,
    pub data: SwapperProviderData,
    pub request: SwapperQuoteRequest,
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
pub struct SwapperProviderData {
    pub provider: SwapperProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<SwapperRoute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapperRoute {
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
pub struct SwapperAssetList {
    pub chains: Vec<Chain>,
    pub asset_ids: Vec<AssetId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapperSwapResult {
    pub status: SwapperSwapStatus,
    pub from_chain: Chain,
    pub from_tx_hash: String,
    pub to_chain: Option<Chain>,
    pub to_tx_hash: Option<String>,
}
