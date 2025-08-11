use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, Chain, SwapProvider};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    pub quote: SwapQuote,
    pub data: SwapQuoteData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct QuoteAsset {
    pub id: String,
    pub symbol: String,
    pub decimals: u32,
}

impl QuoteAsset {
    pub fn asset_id(&self) -> AssetId {
        AssetId::new(&self.id).unwrap()
    }
}

impl QuoteAsset {
    pub fn is_native(&self) -> bool {
        self.asset_id().is_native()
    }

    pub fn chain(&self) -> Chain {
        self.asset_id().chain
    }
}

impl From<AssetId> for QuoteAsset {
    fn from(id: AssetId) -> Self {
        Self {
            id: id.to_string(),
            symbol: String::new(),
            decimals: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuote {
    pub from_value: String,
    pub to_value: String,
    pub provider_data: SwapProviderData,
    pub wallet_address: String,
    pub slippage_bps: u32,
    pub eta_in_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapProviderData {
    pub provider: SwapProvider,
    pub name: String,
    pub protocol_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum SwapStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}
