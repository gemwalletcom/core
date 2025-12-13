use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumIter, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum RewardsEvent {
    CreateUsername,
    Invite,
    Joined,
}

impl RewardsEvent {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn points(&self) -> i32 {
        match self {
            RewardsEvent::CreateUsername => 25,
            RewardsEvent::Invite => 100,
            RewardsEvent::Joined => 10,
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
pub struct RewardsEventItem {
    pub event: RewardsEvent,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}
