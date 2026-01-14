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

    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Unverified | Self::Pending | Self::Verified | Self::Trusted => true,
            Self::Disabled => false,
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
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralCodeActivation {
    pub swap_completed: bool,
    pub swap_amount: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralActivation {
    pub verify_completed: bool,
    pub verify_after: Option<DateTime<Utc>>,
    pub swap_completed: bool,
    pub swap_amount: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct Rewards {
    pub code: Option<String>,
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
    pub redemption_options: Vec<RewardRedemptionOption>,
    pub disable_reason: Option<String>,
    pub referral_allowance: ReferralAllowance,
    pub referral_code_activation: Option<ReferralCodeActivation>,
    pub referral_activation: Option<ReferralActivation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralAllowance {
    pub daily: ReferralQuota,
    pub weekly: ReferralQuota,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ReferralQuota {
    pub limit: i32,
    pub available: i32,
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
