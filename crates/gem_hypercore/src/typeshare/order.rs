use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrder {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: i64,
    pub timestamp: i64,
    pub is_trigger: bool,
    pub trigger_px: Option<String>,
    pub children: Vec<HypercoreOrder>,
    pub is_position_tpsl: bool,
    pub order_type: String,
    pub orig_sz: String,
}