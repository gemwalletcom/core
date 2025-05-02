use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::AssetId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct Markets {
    pub market_cap: f32,
    pub market_cap_change_percentage_24h: f32,

    pub assets: MarketsAssets,
    pub dominance: Vec<MarketDominance>,

    pub total_volume_24h: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct MarketsAssets {
    pub trending: Vec<AssetId>,
    pub gainers: Vec<AssetId>,
    pub losers: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct MarketDominance {
    pub asset_id: String,
    pub dominance: f32,
}
