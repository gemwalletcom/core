use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::referrals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StorageReferral {
    pub address: String,
    pub code: Option<String>,
    pub used_referral_code: Option<String>,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::referrals)]
pub struct NewReferral {
    pub address: String,
    pub code: Option<String>,
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::referrals_uses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReferralUse {
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::referrals_uses)]
pub struct NewReferralUse {
    pub referrer_address: String,
    pub referred_address: String,
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::referrals_events_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReferralEventType {
    pub id: String,
    pub points: i32,
}

impl ReferralEventType {
    pub fn from_primitive(event: primitives::ReferralEvent) -> Self {
        Self {
            id: event.as_ref().to_string(),
            points: event.points(),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::referrals_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReferralEvent {
    pub event_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::referrals_events)]
pub struct NewReferralEvent {
    pub address: String,
    pub event_type: String,
}

impl StorageReferral {
    pub fn as_primitive(&self, referral_count: i32, points: i32) -> primitives::Referral {
        primitives::Referral {
            code: self.code.clone(),
            referral_count,
            points,
            used_referral_code: self.used_referral_code.clone(),
        }
    }
}
