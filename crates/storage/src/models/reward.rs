use chrono::{NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use primitives::rewards::{RedemptionStatus, RewardRedemption, RewardRedemptionOption, RewardRedemptionType};
use primitives::{Asset, RewardEvent, RewardEventType, RewardLevel};
use std::str::FromStr;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardsRow {
    pub username: String,
    pub is_enabled: bool,
    pub level: Option<String>,
    pub points: i32,
    pub referrer_username: Option<String>,
    pub referral_count: i32,
    pub device_id: i32,
    pub verified: bool,
    pub comment: Option<String>,
    pub disable_reason: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards)]
pub struct NewRewardsRow {
    pub username: String,
    pub is_enabled: bool,
    pub level: Option<String>,
    pub points: i32,
    pub referrer_username: Option<String>,
    pub referral_count: i32,
    pub device_id: i32,
    pub verified: bool,
    pub comment: Option<String>,
    pub disable_reason: Option<String>,
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardReferralRow {
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
    pub risk_signal_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
pub struct NewRewardReferralRow {
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
    pub risk_signal_id: i32,
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_events_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardEventTypeRow {
    pub id: String,
    pub points: i32,
}

impl RewardEventTypeRow {
    pub fn from_primitive(event: RewardEventType) -> Self {
        Self {
            id: event.as_ref().to_string(),
            points: event.points(),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardEventRow {
    pub username: String,
    pub event_type: String,
    pub created_at: NaiveDateTime,
}

impl RewardEventRow {
    pub fn as_primitive(&self) -> RewardEvent {
        let event = RewardEventType::from_str(&self.event_type).unwrap();
        RewardEvent {
            points: event.points(),
            event,
            created_at: Utc.from_utc_datetime(&self.created_at),
        }
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_events)]
pub struct NewRewardEventRow {
    pub username: String,
    pub event_type: String,
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_levels_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardLevelTypeRow {
    pub id: String,
}

impl RewardLevelTypeRow {
    pub fn from_primitive(level: RewardLevel) -> Self {
        Self {
            id: level.as_ref().to_string(),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_redemptions_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardRedemptionTypeRow {
    pub id: String,
}

impl RewardRedemptionTypeRow {
    pub fn from_primitive(redemption_type: RewardRedemptionType) -> Self {
        Self {
            id: redemption_type.as_ref().to_string(),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_redemptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardRedemptionRow {
    pub id: i32,
    pub username: String,
    pub option_id: String,
    pub device_id: i32,
    pub transaction_id: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl RewardRedemptionRow {
    pub fn as_primitive(&self, option: RewardRedemptionOption) -> RewardRedemption {
        RewardRedemption {
            id: self.id,
            option,
            status: RedemptionStatus::from_str(&self.status).unwrap_or(RedemptionStatus::Pending),
            transaction_id: self.transaction_id.clone(),
            created_at: Utc.from_utc_datetime(&self.created_at),
        }
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_redemptions)]
pub struct NewRewardRedemptionRow {
    pub username: String,
    pub option_id: String,
    pub device_id: i32,
    pub status: String,
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_redemption_options)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardRedemptionOptionRow {
    pub id: String,
    pub redemption_type: String,
    pub points: i32,
    pub asset_id: Option<String>,
    pub value: String,
    pub remaining: Option<i32>,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

impl RewardRedemptionOptionRow {
    pub fn as_primitive(&self, asset: Option<Asset>) -> RewardRedemptionOption {
        RewardRedemptionOption {
            id: self.id.clone(),
            redemption_type: RewardRedemptionType::from_str(&self.redemption_type).unwrap(),
            points: self.points,
            asset,
            value: self.value.clone(),
            remaining: self.remaining,
        }
    }
}

use crate::models::AssetRow;

#[derive(Debug, Clone)]
pub struct RedemptionOptionFull {
    pub option: RewardRedemptionOptionRow,
    pub asset: Option<AssetRow>,
}

impl RedemptionOptionFull {
    pub fn new(option: RewardRedemptionOptionRow, asset: Option<AssetRow>) -> Self {
        Self { option, asset }
    }

    pub fn as_primitive(&self) -> RewardRedemptionOption {
        self.option.as_primitive(self.asset.as_ref().map(|a| a.as_primitive()))
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_referral_attempts)]
pub struct ReferralAttemptRow {
    pub referrer_username: String,
    pub referred_address: String,
    pub device_id: i32,
    pub risk_signal_id: Option<i32>,
    pub reason: String,
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_risk_signals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RiskSignalRow {
    pub id: i32,
    pub fingerprint: String,
    pub referrer_username: String,
    pub device_id: i32,
    pub device_platform: String,
    pub device_platform_store: String,
    pub device_os: String,
    pub device_model: String,
    pub device_locale: String,
    pub ip_address: String,
    pub ip_country_code: String,
    pub ip_usage_type: String,
    pub ip_isp: String,
    pub ip_abuse_score: i32,
    pub risk_score: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_risk_signals)]
pub struct NewRiskSignalRow {
    pub fingerprint: String,
    pub referrer_username: String,
    pub device_id: i32,
    pub device_platform: String,
    pub device_platform_store: String,
    pub device_os: String,
    pub device_model: String,
    pub device_locale: String,
    pub ip_address: String,
    pub ip_country_code: String,
    pub ip_usage_type: String,
    pub ip_isp: String,
    pub ip_abuse_score: i32,
    pub risk_score: i32,
    pub metadata: Option<serde_json::Value>,
}
