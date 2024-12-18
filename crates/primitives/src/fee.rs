use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

#[typeshare(swift = "Equatable, Sendable, CaseIterable")]
pub enum FeePriority {
    Slow,
    Normal,
    Fast,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, EnumIter)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub enum FeeUnitType {
    SatVb,
    SatB,
    Gwei,
    Native,
}
