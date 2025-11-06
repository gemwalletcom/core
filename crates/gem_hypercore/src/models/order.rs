use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

use crate::models::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub coin: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: UInt64,
    pub is_trigger: bool,
    pub trigger_px: Option<String>,
    pub is_position_tpsl: bool,
    pub orig_sz: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    pub coin: String,
    pub oid: UInt64,
    #[serde(deserialize_with = "serde_serializers::deserialize_option_f64_from_str")]
    pub trigger_px: Option<f64>,
    #[serde(deserialize_with = "serde_serializers::deserialize_option_f64_from_str")]
    pub limit_px: Option<f64>,
    pub is_position_tpsl: bool,
    pub order_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpetualFill {
    pub coin: String,
    pub hash: String,
    pub oid: UInt64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub closed_pnl: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fee: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub px: f64,
    pub dir: String,
}
