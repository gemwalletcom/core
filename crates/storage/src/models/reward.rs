use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardReferral {
    pub referrer_username: String,
    pub referred_username: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_referrals)]
pub struct NewRewardReferral {
    pub referrer_username: String,
    pub referred_username: String,
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_events_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RewardEventType {
    pub id: String,
    pub points: i32,
}

impl RewardEventType {
    pub fn from_primitive(event: primitives::ReferralEvent) -> Self {
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
    pub event_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::rewards_events)]
pub struct NewRewardEvent {
    pub username: String,
    pub event_type: String,
}
