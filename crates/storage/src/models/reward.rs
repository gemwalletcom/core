use chrono::{NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use primitives::{RewardEvent, RewardEventType, RewardLevel};
use std::str::FromStr;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardReferralRow {
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
pub struct NewRewardReferralRow {
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
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
