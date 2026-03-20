use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::{deserialize_f64_from_str, deserialize_option_f64_from_str};
use strum::{Display, EnumString};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display, EnumString)]
#[serde(from = "String", into = "String")]
pub enum FillDirection {
    #[strum(serialize = "Buy")]
    Buy,
    #[strum(serialize = "Sell")]
    Sell,
    #[strum(serialize = "Open Long")]
    OpenLong,
    #[strum(serialize = "Open Short")]
    OpenShort,
    #[strum(serialize = "Close Long")]
    CloseLong,
    #[strum(serialize = "Close Short")]
    CloseShort,
    #[strum(default)]
    Other(String),
}

impl From<String> for FillDirection {
    fn from(value: String) -> Self {
        match value.parse() {
            Ok(direction) => direction,
            Err(_) => FillDirection::Other(value),
        }
    }
}

impl From<FillDirection> for String {
    fn from(value: FillDirection) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFill {
    pub coin: String,
    pub hash: String,
    pub oid: UInt64,
    pub sz: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub closed_pnl: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fee: f64,
    #[serde(default, deserialize_with = "deserialize_option_f64_from_str")]
    pub builder_fee: Option<f64>,
    pub fee_token: Option<String>,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub px: f64,
    pub dir: FillDirection,
    pub time: u64,
    #[serde(default)]
    pub liquidation: Option<Value>,
}
