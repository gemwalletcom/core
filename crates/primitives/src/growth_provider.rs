use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq, Eq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum GrowthProviderType {
    Stake,
    Earn,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq, Eq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum YieldProvider {
    Yo,
}
