use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

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
