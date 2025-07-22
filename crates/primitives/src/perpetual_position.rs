use crate::Perpetual;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum PerpetualMarginType {
    Cross,
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct PriceTarget {
    pub price: Option<f64>,
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct PerpetualPosition {
    pub id: String,
    pub perpetual: Perpetual,
    pub size: f64,
    pub leverage: u8,
    pub liquidation_price: Option<f64>,
    pub margin_type: PerpetualMarginType,
    pub margin_amount: f64,
    pub take_profit: Option<PriceTarget>,
    pub stop_loss: Option<PriceTarget>,
    pub pnl: f64,
}
