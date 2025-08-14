use crate::typeshare::UInt64;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreBalance {
    pub coin: String,
    pub token: u32,
    pub total: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreBalances {
    pub balances: Vec<HypercoreBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreTokens {
    pub tokens: Vec<HypercoreToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreToken {
    pub name: String,
    pub wei_decimals: i32,
    pub index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreStakeBalance {
    pub delegated: String,
    pub undelegated: String,
    pub total_pending_withdrawal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreDelegationBalance {
    pub validator: String,
    pub amount: String,
    pub locked_until_timestamp: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct HypercoreValidator {
    pub validator: String,
    pub name: String,
    pub commission: String,
    pub is_active: bool,
    #[typeshare(skip)]
    pub stats: Vec<(String, HypercoreValidatorStats)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreValidatorStats {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub predicted_apr: f64,
}
