use crate::{AssetId, PerpetualProvider};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct Perpetual {
    pub id: String,
    pub provider: PerpetualProvider,
    pub asset_id: AssetId,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub leverage: Vec<u8>,
}
