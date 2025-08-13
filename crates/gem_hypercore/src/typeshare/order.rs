use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::typeshare::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrder {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: UInt64,
    pub timestamp: UInt64,
    pub is_trigger: bool,
    pub trigger_px: Option<String>,
    pub children: Vec<HypercoreOrder>,
    pub is_position_tpsl: bool,
    pub order_type: String,
    pub orig_sz: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercorePerpetualFill {
    pub coin: String,
    pub hash: String,
    pub oid: UInt64,
    pub closed_pnl: String,
    pub fee: String,
    pub builder_fee: Option<String>,
    pub px: String,
}
