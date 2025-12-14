use crate::DatabaseClient;
use crate::models::{NewRewardEvent, NewRewardReferralRow, RewardEvent, RewardEventTypeRow, RewardReferralRow};
use diesel::prelude::*;

pub trait RewardsEventTypesStore {
    fn add_reward_event_types(&mut self, event_types: Vec<RewardEventTypeRow>) -> Result<usize, diesel::result::Error>;
}

impl RewardsEventTypesStore for DatabaseClient {
    fn add_reward_event_types(&mut self, event_types: Vec<RewardEventTypeRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::rewards_events_types::dsl;
        diesel::insert_into(dsl::rewards_events_types)
            .values(&event_types)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

pub(crate) trait RewardsStore {
    fn add_referral(&mut self, referral: NewRewardReferralRow) -> Result<(), diesel::result::Error>;
    fn get_referrals_by_referrer(&mut self, referrer_username: &str) -> Result<Vec<RewardReferralRow>, diesel::result::Error>;
    fn get_referral_by_referred(&mut self, referred_username: &str) -> Result<Option<RewardReferralRow>, diesel::result::Error>;
    fn get_referral_by_referred_device_id(&mut self, referred_device_id: i32) -> Result<Option<RewardReferralRow>, diesel::result::Error>;
    fn add_event(&mut self, event: NewRewardEvent) -> Result<i32, diesel::result::Error>;
    fn get_event(&mut self, event_id: i32) -> Result<RewardEvent, diesel::result::Error>;
    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEvent>, diesel::result::Error>;
}

impl RewardsStore for DatabaseClient {
    fn add_referral(&mut self, referral: NewRewardReferralRow) -> Result<(), diesel::result::Error> {
        use crate::schema::rewards_referrals::dsl;
        diesel::insert_into(dsl::rewards_referrals).values(&referral).execute(&mut self.connection)?;
        Ok(())
    }

    fn get_referrals_by_referrer(&mut self, referrer_username: &str) -> Result<Vec<RewardReferralRow>, diesel::result::Error> {
        use crate::schema::rewards_referrals::dsl;
        dsl::rewards_referrals
            .filter(dsl::referrer_username.eq(referrer_username))
            .order(dsl::created_at.desc())
            .select(RewardReferralRow::as_select())
            .load(&mut self.connection)
    }

    fn get_referral_by_referred(&mut self, referred_username: &str) -> Result<Option<RewardReferralRow>, diesel::result::Error> {
        use crate::schema::rewards_referrals::dsl;
        dsl::rewards_referrals
            .filter(dsl::referred_username.eq(referred_username))
            .select(RewardReferralRow::as_select())
            .first(&mut self.connection)
            .optional()
    }

    fn get_referral_by_referred_device_id(&mut self, referred_device_id: i32) -> Result<Option<RewardReferralRow>, diesel::result::Error> {
        use crate::schema::rewards_referrals::dsl;
        dsl::rewards_referrals
            .filter(dsl::referred_device_id.eq(referred_device_id))
            .select(RewardReferralRow::as_select())
            .first(&mut self.connection)
            .optional()
    }

    fn add_event(&mut self, event: NewRewardEvent) -> Result<i32, diesel::result::Error> {
        use crate::schema::rewards_events::dsl;
        diesel::insert_into(dsl::rewards_events)
            .values(&event)
            .returning(dsl::id)
            .get_result(&mut self.connection)
    }

    fn get_event(&mut self, event_id: i32) -> Result<RewardEvent, diesel::result::Error> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::id.eq(event_id))
            .select(RewardEvent::as_select())
            .first(&mut self.connection)
    }

    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEvent>, diesel::result::Error> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::username.eq(username))
            .order(dsl::created_at.desc())
            .select(RewardEvent::as_select())
            .load(&mut self.connection)
    }
}
