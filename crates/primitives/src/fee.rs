use num_bigint::BigInt;
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

    pub fn regular<T: Into<BigInt>>(priority: FeePriority, gas_price: T) -> Self {
        Self {
            priority,
            gas_price_type: GasPriceType::regular(gas_price),
        }
    }

    pub fn eip1559<T: Into<BigInt>, U: Into<BigInt>>(priority: FeePriority, gas_price: T, priority_fee: U) -> Self {
        Self {
            priority,
            gas_price_type: GasPriceType::eip1559(gas_price, priority_fee),
        }
    }
}
