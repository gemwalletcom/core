use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable, CaseIterable")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ReferralEvent {
    Invite,
    Joined,
}

impl ReferralEvent {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn points(&self) -> i32 {
        match self {
            ReferralEvent::Invite => 100,
            ReferralEvent::Joined => 10,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Referral {
    pub code: Option<String>,
    pub referral_count: i32,
    pub points: i32,
    pub used_referral_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralCodeRequest {
    pub address: String,
    pub message: String,
    pub signature: String,
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralEventItem {
    pub event: ReferralEvent,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}
