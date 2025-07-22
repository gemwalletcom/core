use crate::{AssetId, PerpetualPosition, PerpetualProvider};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Perpetual {
    pub id: String,
    pub name: String,
    pub provider: PerpetualProvider,
    pub asset_id: AssetId,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub funding: f64,
    pub leverage: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct PerpetualPositionData {
    pub perpetual: Perpetual,
    pub positions: Vec<PerpetualPosition>,
}
