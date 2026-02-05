use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use crate::Chain;

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
    pub fee: f64,
    pub apy: f64,
    pub provider_type: EarnProviderType,
}

impl EarnProvider {
    pub fn new_stake(chain: Chain, id: String, name: String, is_active: bool, commission: f64, apr: f64) -> Self {
        Self {
            chain,
            id,
            name,
            is_active,
            fee: commission,
            apy: apr,
            provider_type: EarnProviderType::Stake,
        }
    }

    pub fn new_yield(chain: Chain, id: String, name: String, is_active: bool, fee: f64, apy: f64) -> Self {
        Self {
            chain,
            id,
            name,
            is_active,
            fee,
            apy,
            provider_type: EarnProviderType::Yield,
        }
    }
}
