use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, Price};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlert {
    pub asset_id: String,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlertData {
    pub asset: Asset,
    pub price: Option<Price>,
    pub price_alert: PriceAlert,
}

pub type PriceAlerts = Vec<PriceAlert>;
