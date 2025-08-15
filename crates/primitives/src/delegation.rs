use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use crate::{AssetId, Chain, Price};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    pub base: DelegationBase,
    pub validator: DelegationValidator,
    pub price: Option<Price>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct DelegationBase {
    pub asset_id: AssetId,
    pub state: DelegationState,
    pub balance: String,
    pub shares: String,
    pub rewards: String,
    pub completion_date: Option<DateTime<Utc>>,
    pub delegation_id: String,
    pub validator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct DelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commision: f64,
    pub apr: f64,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DelegationState {
    Active,
    Pending,
    Undelegating,
    Inactive,
    Activating,
    Deactivating,
    AwaitingWithdrawal,
}
