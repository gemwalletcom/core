use crate::DatabaseClient;
use crate::models::{
    NewRewardEventRow, NewRewardRedemptionRow, NewRewardReferralRow, RewardEventRow, RewardEventTypeRow, RewardRedemptionOptionRow, RewardRedemptionRow,
    RewardRedemptionTypeRow, RewardReferralRow,
};
use diesel::prelude::*;

#[derive(Debug, Clone)]
pub enum RedemptionUpdate {
    Status(String),
    TransactionId(String),
}

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

pub trait RewardsRedemptionTypesStore {
    fn add_reward_redemption_types(&mut self, redemption_types: Vec<RewardRedemptionTypeRow>) -> Result<usize, diesel::result::Error>;
}

impl RewardsRedemptionTypesStore for DatabaseClient {
    fn add_reward_redemption_types(&mut self, redemption_types: Vec<RewardRedemptionTypeRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::rewards_redemptions_types::dsl;
        diesel::insert_into(dsl::rewards_redemptions_types)
            .values(&redemption_types)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

pub trait RewardsRedemptionOptionsStore {
    fn add_redemption_options(&mut self, options: Vec<RewardRedemptionOptionRow>) -> Result<usize, diesel::result::Error>;
}

impl RewardsRedemptionOptionsStore for DatabaseClient {
    fn add_redemption_options(&mut self, options: Vec<RewardRedemptionOptionRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::rewards_redemption_options::dsl;
        diesel::insert_into(dsl::rewards_redemption_options)
            .values(&options)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

pub(crate) trait RewardsStore {
    fn add_referral(&mut self, referral: NewRewardReferralRow) -> Result<(), diesel::result::Error>;
    fn get_referral_by_referred_device_id(&mut self, referred_device_id: i32) -> Result<Option<RewardReferralRow>, diesel::result::Error>;
    fn add_event_with_points(&mut self, event: NewRewardEventRow, points: i32) -> Result<i32, diesel::result::Error>;
    fn get_event(&mut self, event_id: i32) -> Result<RewardEventRow, diesel::result::Error>;
    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEventRow>, diesel::result::Error>;
    fn add_redemption_with_points_deduction(&mut self, username: &str, points: i32, redemption: NewRewardRedemptionRow) -> Result<i32, diesel::result::Error>;
    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), diesel::result::Error>;
    fn get_redemption(&mut self, redemption_id: i32) -> Result<RewardRedemptionRow, diesel::result::Error>;
    fn get_redemption_options(&mut self) -> Result<Vec<RewardRedemptionOptionRow>, diesel::result::Error>;
    fn get_redemption_option(&mut self, id: &str) -> Result<RewardRedemptionOptionRow, diesel::result::Error>;
}

impl RewardsStore for DatabaseClient {
    fn add_referral(&mut self, referral: NewRewardReferralRow) -> Result<(), diesel::result::Error> {
        use crate::schema::{rewards_referrals, usernames};
        use diesel::Connection;

        self.connection.transaction(|conn| {
            diesel::insert_into(rewards_referrals::table).values(&referral).execute(conn)?;

            diesel::update(usernames::table.filter(usernames::username.eq(&referral.referred_username)))
                .set(usernames::referrer_username.eq(&referral.referrer_username))
                .execute(conn)?;

            diesel::update(usernames::table.filter(usernames::username.eq(&referral.referrer_username)))
                .set(usernames::referral_count.eq(usernames::referral_count + 1))
                .execute(conn)?;

            Ok(())
        })
    }

    fn get_referral_by_referred_device_id(&mut self, referred_device_id: i32) -> Result<Option<RewardReferralRow>, diesel::result::Error> {
        use crate::schema::rewards_referrals::dsl;
        dsl::rewards_referrals
            .filter(dsl::referred_device_id.eq(referred_device_id))
            .select(RewardReferralRow::as_select())
            .first(&mut self.connection)
            .optional()
    }

    fn add_event_with_points(&mut self, event: NewRewardEventRow, points: i32) -> Result<i32, diesel::result::Error> {
        use crate::schema::{rewards_events, usernames};
        use diesel::Connection;

        if points < 0 {
            return Err(diesel::result::Error::RollbackTransaction);
        }

        self.connection.transaction(|conn| {
            let event_id = diesel::insert_into(rewards_events::table)
                .values(&event)
                .returning(rewards_events::id)
                .get_result(conn)?;

            let affected = diesel::update(usernames::table.filter(usernames::username.eq(&event.username)))
                .set(usernames::points.eq(usernames::points + points))
                .execute(conn)?;

            if affected == 0 {
                return Err(diesel::result::Error::NotFound);
            }

            Ok(event_id)
        })
    }

    fn get_event(&mut self, event_id: i32) -> Result<RewardEventRow, diesel::result::Error> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::id.eq(event_id))
            .select(RewardEventRow::as_select())
            .first(&mut self.connection)
    }

    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEventRow>, diesel::result::Error> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::username.eq(username))
            .order(dsl::created_at.desc())
            .select(RewardEventRow::as_select())
            .load(&mut self.connection)
    }

    fn add_redemption_with_points_deduction(&mut self, username: &str, points: i32, redemption: NewRewardRedemptionRow) -> Result<i32, diesel::result::Error> {
        use crate::schema::{rewards_redemptions, usernames};
        use diesel::Connection;

        if points <= 0 {
            return Err(diesel::result::Error::RollbackTransaction);
        }

        self.connection.transaction(|conn| {
            let affected = diesel::update(usernames::table.filter(usernames::username.eq(username).and(usernames::points.ge(points))))
                .set(usernames::points.eq(usernames::points - points))
                .execute(conn)?;

            if affected == 0 {
                return Err(diesel::result::Error::NotFound);
            }

            diesel::insert_into(rewards_redemptions::table)
                .values(&redemption)
                .returning(rewards_redemptions::id)
                .get_result(conn)
        })
    }

    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), diesel::result::Error> {
        use crate::schema::rewards_redemptions::dsl;

        if updates.is_empty() {
            return Ok(());
        }

        for update in updates {
            let target = dsl::rewards_redemptions.find(redemption_id);
            match update {
                RedemptionUpdate::Status(value) => diesel::update(target).set(dsl::status.eq(value)).execute(&mut self.connection)?,
                RedemptionUpdate::TransactionId(value) => diesel::update(target).set(dsl::transaction_id.eq(value)).execute(&mut self.connection)?,
            };
        }

        Ok(())
    }

    fn get_redemption(&mut self, redemption_id: i32) -> Result<RewardRedemptionRow, diesel::result::Error> {
        use crate::schema::rewards_redemptions::dsl;
        dsl::rewards_redemptions
            .filter(dsl::id.eq(redemption_id))
            .select(RewardRedemptionRow::as_select())
            .first(&mut self.connection)
    }

    fn get_redemption_options(&mut self) -> Result<Vec<RewardRedemptionOptionRow>, diesel::result::Error> {
        use crate::schema::rewards_redemption_options::dsl;
        dsl::rewards_redemption_options
            .select(RewardRedemptionOptionRow::as_select())
            .load(&mut self.connection)
    }

    fn get_redemption_option(&mut self, id: &str) -> Result<RewardRedemptionOptionRow, diesel::result::Error> {
        use crate::schema::rewards_redemption_options::dsl;
        dsl::rewards_redemption_options
            .filter(dsl::id.eq(id))
            .select(RewardRedemptionOptionRow::as_select())
            .first(&mut self.connection)
    }
}
