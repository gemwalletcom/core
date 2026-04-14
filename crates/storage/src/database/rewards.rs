use crate::DatabaseClient;
use crate::models::{NewRewardEventRow, NewRewardsRow, RewardEventRow, RewardsRow};
use crate::sql_types::{RewardEventType, RewardStatus};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use primitives::RewardStatus as PrimitiveRewardStatus;

#[derive(Debug, Clone)]
pub enum RewardsUpdate {
    Status(RewardStatus),
    VerifyAfter(NaiveDateTime),
    ClearVerifyAfter,
}

#[derive(Debug, Clone)]
pub enum RewardsFilter {
    Username(String),
    Statuses(Vec<PrimitiveRewardStatus>),
    Limit(i64),
}

pub(crate) trait RewardsStore {
    fn get_rewards_by_filter(&mut self, filters: Vec<RewardsFilter>) -> Result<Vec<RewardsRow>, DieselError>;
    fn create_rewards(&mut self, rewards: NewRewardsRow) -> Result<RewardsRow, DieselError>;
    fn update_rewards(&mut self, username: &str, update: RewardsUpdate) -> Result<usize, DieselError>;
    fn add_event(&mut self, event: NewRewardEventRow, points: i32) -> Result<RewardEventRow, DieselError>;
    fn get_event(&mut self, event_id: i32) -> Result<RewardEventRow, DieselError>;
    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEventRow>, DieselError>;
    fn get_top_referrers_since(&mut self, event_types: &[RewardEventType], since: NaiveDateTime, limit: i64) -> Result<Vec<(String, i64)>, DieselError>;
    fn disable_rewards(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DieselError>;
}

impl RewardsStore for DatabaseClient {
    fn get_rewards_by_filter(&mut self, filters: Vec<RewardsFilter>) -> Result<Vec<RewardsRow>, DieselError> {
        use crate::schema::rewards::dsl;
        let mut query = dsl::rewards.into_boxed();

        for filter in filters {
            match filter {
                RewardsFilter::Username(username) => {
                    query = query.filter(dsl::username.eq(username));
                }
                RewardsFilter::Statuses(statuses) => {
                    query = query.filter(dsl::status.eq_any(statuses.into_iter().map(RewardStatus::from).collect::<Vec<_>>()));
                }
                RewardsFilter::Limit(limit) => {
                    query = query.limit(limit);
                }
            }
        }

        query.select(RewardsRow::as_select()).load(&mut self.connection)
    }

    fn create_rewards(&mut self, rewards: NewRewardsRow) -> Result<RewardsRow, DieselError> {
        use crate::schema::rewards::dsl;
        diesel::insert_into(dsl::rewards)
            .values(&rewards)
            .returning(RewardsRow::as_returning())
            .get_result(&mut self.connection)
    }

    fn update_rewards(&mut self, username: &str, update: RewardsUpdate) -> Result<usize, DieselError> {
        use crate::schema::rewards::dsl;
        let target = dsl::rewards.filter(dsl::username.eq(username));
        match update {
            RewardsUpdate::Status(status) => diesel::update(target).set(dsl::status.eq(status)).execute(&mut self.connection),
            RewardsUpdate::VerifyAfter(dt) => diesel::update(target).set(dsl::verify_after.eq(dt)).execute(&mut self.connection),
            RewardsUpdate::ClearVerifyAfter => diesel::update(target).set(dsl::verify_after.eq(None::<NaiveDateTime>)).execute(&mut self.connection),
        }
    }

    fn add_event(&mut self, new_event: NewRewardEventRow, points: i32) -> Result<RewardEventRow, DieselError> {
        use crate::schema::{rewards, rewards_events};
        use diesel::Connection;

        if points < 0 {
            return Err(DieselError::RollbackTransaction);
        }

        self.connection.transaction(|conn| {
            let event = diesel::insert_into(rewards_events::table)
                .values(&new_event)
                .returning(RewardEventRow::as_returning())
                .get_result(conn)?;

            diesel::update(rewards::table.filter(rewards::username.eq(&new_event.username)))
                .set(rewards::points.eq(rewards::points + points))
                .returning(rewards::username)
                .get_result::<String>(conn)?;

            Ok(event)
        })
    }

    fn get_event(&mut self, event_id: i32) -> Result<RewardEventRow, DieselError> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::id.eq(event_id))
            .select(RewardEventRow::as_select())
            .first(&mut self.connection)
    }

    fn get_events(&mut self, username: &str) -> Result<Vec<RewardEventRow>, DieselError> {
        use crate::schema::rewards_events::dsl;
        dsl::rewards_events
            .filter(dsl::username.eq(username))
            .order(dsl::created_at.desc())
            .select(RewardEventRow::as_select())
            .load(&mut self.connection)
    }

    fn get_top_referrers_since(&mut self, event_types: &[RewardEventType], since: NaiveDateTime, limit: i64) -> Result<Vec<(String, i64)>, DieselError> {
        use crate::schema::{rewards, rewards_events};
        use diesel::dsl::count_star;

        rewards_events::table
            .inner_join(rewards::table.on(rewards_events::username.eq(rewards::username)))
            .filter(rewards::status.ne(RewardStatus::Disabled))
            .filter(rewards_events::event_type.eq_any(event_types))
            .filter(rewards_events::created_at.ge(since))
            .group_by(rewards_events::username)
            .select((rewards_events::username, count_star()))
            .order_by(count_star().desc())
            .limit(limit)
            .load(&mut self.connection)
    }

    fn disable_rewards(&mut self, username: &str, reason: &str, comment: &str) -> Result<i32, DieselError> {
        use crate::schema::{rewards, rewards_events};
        use diesel::Connection;

        self.connection.transaction(|conn| {
            diesel::update(rewards::table.filter(rewards::username.eq(username)))
                .set((rewards::status.eq(RewardStatus::Disabled), rewards::disable_reason.eq(reason), rewards::comment.eq(comment)))
                .execute(conn)?;

            let event_id = diesel::insert_into(rewards_events::table)
                .values(NewRewardEventRow {
                    username: username.to_string(),
                    event_type: RewardEventType::Disabled,
                })
                .returning(rewards_events::id)
                .get_result(conn)?;

            Ok(event_id)
        })
    }
}
