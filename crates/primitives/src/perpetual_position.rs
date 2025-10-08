use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{AssetId, PerpetualDirection};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PerpetualMarginType {
    Cross,
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PerpetualOrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct PerpetualTriggerOrder {
    pub price: f64,
    pub order_type: PerpetualOrderType,
    pub order_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PerpetualPosition {
    pub id: String,
    pub perpetual_id: String,
    pub asset_id: AssetId,
    pub size: f64,
    pub size_value: f64,
    pub leverage: u8,
    pub entry_price: Option<f64>,
    pub liquidation_price: Option<f64>,
    pub margin_type: PerpetualMarginType,
    pub direction: PerpetualDirection,
    pub margin_amount: f64,
    pub take_profit: Option<PerpetualTriggerOrder>,
    pub stop_loss: Option<PerpetualTriggerOrder>,
    pub pnl: f64,
    pub funding: Option<f32>,
}
