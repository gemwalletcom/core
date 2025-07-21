use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreAssetPositions {
    pub asset_positions: Vec<HypercoreAssetPosition>,
    pub margin_summary: HypercoreMarginSummary,
    pub cross_margin_summary: HypercoreMarginSummary,
    pub cross_maintenance_margin_used: String,
    pub withdrawable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreMarginSummary {
    pub account_value: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
    pub total_margin_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct HypercoreAssetPosition {
    #[serde(rename = "type")]
    pub position_type: HypercorePositionType,
    pub position: HypercorePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub enum HypercorePositionType {
    OneWay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercorePosition {
    pub coin: String,
    pub szi: String,
    pub leverage: HypercoreLeverage,
    pub entry_px: String,
    pub position_value: String,
    pub unrealized_pnl: String,
    pub return_on_equity: String,
    pub liquidation_px: String,
    pub margin_used: String,
    pub max_leverage: u32,
    pub cum_funding: HypercoreCumulativeFunding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct HypercoreLeverage {
    #[serde(rename = "type")]
    pub leverage_type: HypercoreLeverageType,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub enum HypercoreLeverageType {
    Cross,
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreCumulativeFunding {
    pub all_time: String,
    pub since_open: String,
    pub since_change: String,
}
