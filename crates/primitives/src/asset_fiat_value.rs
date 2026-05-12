use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetFiatValue {
    pub amount: f64,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}
