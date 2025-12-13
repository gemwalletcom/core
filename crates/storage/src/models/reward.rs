use chrono::{NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use primitives::RewardsEventItem;
use std::str::FromStr;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardReferral {
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
pub struct NewRewardReferral {
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_events_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardEventType {
    pub id: String,
    pub points: i32,
}

impl RewardEventType {
    pub fn from_primitive(event: primitives::RewardsEvent) -> Self {
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
    pub fn as_primitive(&self) -> RewardsEventItem {
        let event = primitives::RewardsEvent::from_str(&self.event_type).unwrap();
        RewardsEventItem {
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
