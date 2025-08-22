use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

pub use crate::gas_price_type::GasPriceType;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum FeePriority {
    Slow,
    Normal,
    Fast,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, EnumIter)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub enum FeeUnitType {
    SatVb,
    Gwei,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeRate {
    pub priority: FeePriority,
    pub gas_price_type: GasPriceType,
}

impl FeeRate {
    pub fn new(priority: FeePriority, gas_price_type: GasPriceType) -> Self {
        Self { priority, gas_price_type }
    }
}
