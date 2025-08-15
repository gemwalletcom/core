use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum FeePriority {
    Slow,
    Normal,
    Fast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeePriorityValue {
    pub priority: FeePriority,
    pub value: String,
}

impl FeePriorityValue {
    pub fn new(priority: FeePriority, value: String) -> Self {
        Self { priority, value }
    }
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
