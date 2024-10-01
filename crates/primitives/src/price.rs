use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, AssetDetailsInfo, PriceAlert};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price: f64,
    pub price_change_percentage_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceFull {
    pub asset_id: String,
    pub coin_id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
struct PriceData {
    asset: Asset,
    price: Option<Price>,
    price_alert: Option<PriceAlert>,
    details: Option<AssetDetailsInfo>,
}
