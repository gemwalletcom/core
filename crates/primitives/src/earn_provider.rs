use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use crate::{Chain, StakeValidator};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq, Eq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum YieldProvider {
    Yo,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq, Eq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum EarnProviderType {
    Stake,
    Yield,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EarnProvider {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commission: f64,
    pub apr: f64,
    pub provider_type: EarnProviderType,
}

impl EarnProvider {
    pub fn new(chain: Chain, id: String, name: String, is_active: bool, commission: f64, apr: f64, provider_type: EarnProviderType) -> Self {
        Self {
            chain,
            id,
            name,
            is_active,
            commission,
            apr,
            provider_type,
        }
    }
}

impl From<EarnProvider> for StakeValidator {
    fn from(value: EarnProvider) -> Self {
        StakeValidator::new(value.id, value.name)
    }
}
