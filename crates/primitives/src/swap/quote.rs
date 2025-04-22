use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, Chain};

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
#[typeshare]
pub struct QuoteRequest {
    pub from_address: String,
    pub to_address: String,
    pub from_asset: QuoteAsset,
    pub to_asset: QuoteAsset,
    pub from_value: String,
    pub referral_bps: u32,
    pub slippage_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct Quote {
    pub quote: QuoteRequest,
    pub output_value: String,
    pub output_min_value: String,
    pub route_data: serde_json::Value,
    pub eta_in_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct QuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub limit: Option<String>,
}
