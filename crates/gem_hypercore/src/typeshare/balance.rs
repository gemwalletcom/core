use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::typeshare::UInt64;

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
