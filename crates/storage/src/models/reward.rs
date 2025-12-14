use chrono::{NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
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
    pub fn from_primitive(event: primitives::RewardEventType) -> Self {
        Self {
            id: event.as_ref().to_string(),
            points: event.points(),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardEvent {
    pub username: String,
    pub event_type: String,
    pub created_at: NaiveDateTime,
}

impl RewardEvent {
    pub fn as_primitive(&self) -> primitives::RewardEvent {
        let event = primitives::RewardEventType::from_str(&self.event_type).unwrap();
        primitives::RewardEvent {
            points: event.points(),
            event,
            created_at: Utc.from_utc_datetime(&self.created_at),
        }
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_events)]
pub struct NewRewardEvent {
    pub username: String,
    pub event_type: String,
}
