use super::permit2_data::Permit2Data;
use crate::config::swap_config::{SwapReferralFees, DEFAULT_SLIPPAGE_BPS};
use primitives::{AssetId, Chain};
use std::fmt::Debug;
use std::str::FromStr;

use super::remote_models::*;

#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub struct SwapProviderConfig(SwapperProviderType);

impl SwapProviderConfig {
    pub fn id(&self) -> SwapperProvider {
        self.0.id
    }
}

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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
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
            SwapperProvider::Thorchain => SwapperProviderMode::OmniChain(vec![Chain::Thorchain]),
            SwapperProvider::Relay => SwapperProviderMode::OmniChain(vec![Chain::Hyperliquid, Chain::Manta, Chain::Berachain]),
            SwapperProvider::Across => SwapperProviderMode::Bridge,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapperQuoteRequest {
    pub from_asset: SwapperQuoteAsset,
    pub to_asset: SwapperQuoteAsset,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: String,
    pub mode: SwapperMode,
    pub options: SwapperOptions,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapperOptions {
    pub slippage: SwapperSlippage,
    pub fee: Option<SwapReferralFees>,
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapperQuote {
    pub from_value: String,
    pub to_value: String,
    pub data: SwapperProviderData,
    pub request: SwapperQuoteRequest,
    pub eta_in_seconds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum ApprovalType {
    Approve(SwapperApprovalData),
    Permit2(Permit2ApprovalData),
    None,
}

impl ApprovalType {
    pub fn approval_data(&self) -> Option<SwapperApprovalData> {
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
pub struct Permit2ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub permit2_contract: String,
    pub permit2_nonce: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapperProviderData {
    pub provider: SwapperProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<SwapperRoute>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapperRoute {
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

#[derive(Debug, Clone, uniffi::Record, PartialEq)]
pub struct SwapperAssetList {
    pub chains: Vec<Chain>,
    pub asset_ids: Vec<AssetId>,
}
