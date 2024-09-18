use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{Asset, Price};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlert {
    pub asset_id: String,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
    pub price_direction: Option<PriceDirection>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlertData {
    pub asset: Asset,
    pub price: Option<Price>,
    pub price_alert: PriceAlert,
}

#[derive(Clone, Debug, Serialize, Deserialize, AsRefStr, EnumString, PartialEq)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PriceDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PriceAlertType {
    PriceChangesUp,
    PriceChangesDown,
    PriceUp,
    PriceDown,
    PricePercentChangeUp,
    PricePercentChangeDown,
}

pub type PriceAlerts = Vec<PriceAlert>;
