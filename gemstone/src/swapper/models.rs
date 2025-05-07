use super::permit2_data::Permit2Data;
use crate::config::swap_config::{SwapReferralFees, DEFAULT_SLIPPAGE_BPS};
use primitives::{AssetId, Chain};
use std::fmt::Debug;
use std::str::FromStr;

use super::remote_models::*;

#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub struct SwapProviderConfig(SwapProviderType);

impl SwapProviderConfig {
    pub fn id(&self) -> GemSwapProvider {
        self.0.id
    }
}

#[uniffi::export]
impl SwapProviderConfig {
    #[uniffi::constructor]
    pub fn new(id: GemSwapProvider) -> Self {
        Self(SwapProviderType::new(id))
    }
    #[uniffi::constructor]
    pub fn from_string(id: String) -> Self {
        let id = GemSwapProvider::from_str(&id).unwrap();
        Self(SwapProviderType::new(id))
    }
    pub fn inner(&self) -> SwapProviderType {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SwapProviderType {
    pub id: GemSwapProvider,
    pub name: String,
    pub protocol: String,
    pub protocol_id: String,
}

impl SwapProviderType {
    pub fn new(id: GemSwapProvider) -> Self {
        Self {
            id,
            name: id.name().to_string(),
            protocol: id.protocol_name().to_string(),
            protocol_id: id.id().to_string(),
        }
    }

    pub fn mode(&self) -> GemSwapProviderMode {
        match self.id {
            GemSwapProvider::UniswapV3
            | GemSwapProvider::UniswapV4
            | GemSwapProvider::PancakeswapV3
            | GemSwapProvider::PancakeswapAptosV2
            | GemSwapProvider::Orca
            | GemSwapProvider::Jupiter
            | GemSwapProvider::Oku
            | GemSwapProvider::Wagmi
            | GemSwapProvider::Cetus
            | GemSwapProvider::StonfiV2
            | GemSwapProvider::Reservoir
            | GemSwapProvider::Symbiosis => GemSwapProviderMode::OnChain,
            GemSwapProvider::Mayan => GemSwapProviderMode::CrossChain,
            GemSwapProvider::Thorchain => GemSwapProviderMode::OmniChain(Chain::Thorchain),
            GemSwapProvider::Across => GemSwapProviderMode::Bridge,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapQuoteRequest {
    pub from_asset: GemQuoteAsset,
    pub to_asset: GemQuoteAsset,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: String,
    pub mode: GemSwapMode,
    pub options: GemSwapOptions,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapOptions {
    pub slippage: GemSlippage,
    pub fee: Option<SwapReferralFees>,
    pub preferred_providers: Vec<GemSwapProvider>,
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
    pub request: SwapQuoteRequest,
    pub eta_in_seconds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
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
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapProviderData {
    pub provider: SwapProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<SwapRoute>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapRoute {
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
