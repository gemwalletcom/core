use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::Asset;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumIter, EnumString, AsRefStr, PartialEq)]
//#[typeshare(swift = "Equatable, Hashable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum RewardLevel {}

impl RewardLevel {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumIter, EnumString, AsRefStr, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum RewardRedemptionType {
    Asset,
}

impl RewardRedemptionType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumIter, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum RewardEventType {
    CreateUsername,
    InviteNew,
    InviteExisting,
    Joined,
}

impl RewardEventType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn points(&self) -> i32 {
        match self {
            Self::CreateUsername => 25,
            Self::InviteNew => 100,
            Self::InviteExisting => 10,
            Self::Joined => 10,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Rewards {
    pub code: Option<String>,
    pub referral_count: i32,
    pub points: i32,
    pub used_referral_code: Option<String>,
    pub is_enabled: bool,
    pub redemption_options: Vec<RewardRedemptionOption>,
    //pub level: Option<RewardLevel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralCode {
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct RewardEvent {
    pub event: RewardEventType,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct RewardRedemption {
    pub id: i32,
    pub option: RewardRedemptionOption,
    pub status: RedemptionStatus,
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum RedemptionStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct RewardRedemptionOption {
    pub id: String,
    pub redemption_type: RewardRedemptionType,
    pub points: i32,
    pub asset: Option<Asset>,
    pub value: String,
    pub remaining: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct RedemptionRequest {
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct RedemptionResult {
    pub redemption: RewardRedemption,
}

#[derive(Clone, Debug)]
pub struct RedemptionResponse {
    pub result: RedemptionResult,
    pub redemption_id: i32,
}
