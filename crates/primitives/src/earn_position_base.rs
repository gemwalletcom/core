use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use crate::AssetId;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum EarnPositionState {
    Active,
    Pending,
    Inactive,
    Activating,
    Deactivating,
    AwaitingWithdrawal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EarnPositionBase {
    pub asset_id: AssetId,
    pub state: EarnPositionState,
    pub balance: BigUint,
    pub shares: BigUint,
    pub rewards: BigUint,
    pub unlock_date: Option<DateTime<Utc>>,
    pub position_id: String,
    pub provider_id: String,
}
