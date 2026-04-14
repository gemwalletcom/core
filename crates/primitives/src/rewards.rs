use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::Asset;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumIter, AsRefStr, PartialEq)]
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
    GiftAsset,
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
#[derive(Default)]
pub enum RewardStatus {
    #[default]
    Unverified,
    Pending,
    Verified,
    Trusted,
    Disabled,
}

impl RewardStatus {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn is_verified(&self) -> bool {
        match self {
            Self::Verified | Self::Trusted => true,
            Self::Unverified | Self::Pending | Self::Disabled => false,
        }
    }

    pub fn is_disabled(&self) -> bool {
        match self {
            Self::Disabled => true,
            Self::Unverified | Self::Pending | Self::Verified | Self::Trusted => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumIter, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum RewardEventType {
    CreateUsername,
    InvitePending,
    InviteNew,
    InviteExisting,
    Joined,
    Disabled,
    Redeemed,
}

impl RewardEventType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn points(&self) -> i32 {
        match self {
            Self::CreateUsername => 25,
            Self::InvitePending => 0,
            Self::InviteNew => 100,
            Self::InviteExisting => 10,
            Self::Joined => 10,
            Self::Disabled => 0,
            Self::Redeemed => 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Rewards {
    pub code: Option<String>,
    #[typeshare(skip)]
    pub invite_reward_points: i32,
    pub referral_count: i32,
    pub points: i32,
    pub used_referral_code: Option<String>,
    pub status: RewardStatus,
    #[typeshare(skip)]
    pub is_enabled: bool,
    #[typeshare(skip)]
    pub verified: bool,
    #[typeshare(skip)]
    #[serde(skip)]
    pub created_at: chrono::NaiveDateTime,
    pub verify_after: Option<DateTime<Utc>>,
    pub redemption_options: Vec<RewardRedemptionOption>,
    pub disable_reason: Option<String>,
}

impl Default for Rewards {
    fn default() -> Self {
        Self {
            code: None,
            invite_reward_points: RewardEventType::InviteNew.points(),
            referral_count: 0,
            points: 0,
            used_referral_code: None,
            status: RewardStatus::Unverified,
            is_enabled: false,
            verified: false,
            created_at: chrono::DateTime::<Utc>::UNIX_EPOCH.naive_utc(),
            verify_after: None,
            redemption_options: vec![],
            disable_reason: None,
        }
    }
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
    #[typeshare(skip)]
    #[serde(skip)]
    pub username: String,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, EnumString, EnumIter, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum RedemptionStatus {
    Pending,
    Processing,
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralLeader {
    pub username: String,
    pub referrals: i32,
    pub points: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralLeaderboard {
    pub daily: Vec<ReferralLeader>,
    pub weekly: Vec<ReferralLeader>,
    pub monthly: Vec<ReferralLeader>,
}

#[cfg(test)]
mod tests {
    use super::RewardStatus;

    #[test]
    fn test_reward_status() {
        assert!(!RewardStatus::Unverified.is_verified());
        assert!(!RewardStatus::Pending.is_verified());
        assert!(RewardStatus::Verified.is_verified());
        assert!(RewardStatus::Trusted.is_verified());
        assert!(!RewardStatus::Disabled.is_verified());

        assert!(!RewardStatus::Unverified.is_disabled());
        assert!(!RewardStatus::Pending.is_disabled());
        assert!(!RewardStatus::Verified.is_disabled());
        assert!(!RewardStatus::Trusted.is_disabled());
        assert!(RewardStatus::Disabled.is_disabled());
    }
}
