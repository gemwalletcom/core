use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::referral::ReferralInfo;
use primitives::{AssetId, Chain};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct QuoteAsset {
    #[serde(skip)]
    pub id: AssetId,
    pub asset_id: String,
    pub symbol: String,
    pub decimals: u32,
}

impl QuoteAsset {
    pub fn is_native(&self) -> bool {
        self.id.is_native()
    }

    pub fn chain(&self) -> Chain {
        self.id.chain
    }
}

impl From<AssetId> for QuoteAsset {
    fn from(id: AssetId) -> Self {
        Self {
            id: id.clone(),
            asset_id: id.to_string(),
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
    pub referral: ReferralInfo,
    pub slippage_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct Quote {
    pub quote: QuoteRequest,
    pub output_value: String,
    pub output_min_value: String,
    pub route_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct QuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
}
