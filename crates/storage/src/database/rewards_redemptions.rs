use crate::DatabaseClient;
use crate::models::{AssetRow, NewRewardRedemptionRow, RedemptionOptionFull, RewardRedemptionOptionRow, RewardRedemptionRow, RewardRedemptionTypeRow};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

#[derive(Debug, Clone)]
pub enum RedemptionUpdate {
    Status(String),
    TransactionId(String),
    Error(String),
}

pub trait RewardsRedemptionTypesStore {
    fn add_reward_redemption_types(&mut self, redemption_types: Vec<RewardRedemptionTypeRow>) -> Result<usize, DieselError>;
}

impl RewardsRedemptionTypesStore for DatabaseClient {
    fn add_reward_redemption_types(&mut self, redemption_types: Vec<RewardRedemptionTypeRow>) -> Result<usize, DieselError> {
        use crate::schema::rewards_redemptions_types::dsl;
        diesel::insert_into(dsl::rewards_redemptions_types)
            .values(&redemption_types)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

pub(crate) trait RewardsRedemptionsStore {
    fn add_redemption(&mut self, username: &str, points: i32, redemption: NewRewardRedemptionRow) -> Result<i32, DieselError>;
    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DieselError>;
    fn get_redemption(&mut self, redemption_id: i32) -> Result<RewardRedemptionRow, DieselError>;
    fn get_redemption_options(&mut self, types: &[String]) -> Result<Vec<RedemptionOptionFull>, DieselError>;
    fn get_redemption_option(&mut self, id: &str) -> Result<RedemptionOptionFull, DieselError>;
    fn count_redemptions_since(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DieselError>;
}

impl RewardsRedemptionsStore for DatabaseClient {
    fn add_redemption(&mut self, username: &str, points: i32, redemption: NewRewardRedemptionRow) -> Result<i32, DieselError> {
        use crate::schema::{rewards, rewards_redemption_options, rewards_redemptions};
        use diesel::Connection;

        if points < 0 {
            return Err(DieselError::RollbackTransaction);
        }

        self.connection.transaction(|conn| {
            let rows_updated = diesel::update(
                rewards_redemption_options::table.filter(
                    rewards_redemption_options::id
                        .eq(&redemption.option_id)
                        .and(rewards_redemption_options::remaining.is_null().or(rewards_redemption_options::remaining.gt(0))),
                ),
            )
            .set(rewards_redemption_options::remaining.eq(rewards_redemption_options::remaining - 1))
            .execute(conn)?;

            if rows_updated == 0 {
                return Err(DieselError::NotFound);
            }

            if points > 0 {
                diesel::update(rewards::table.filter(rewards::username.eq(username).and(rewards::points.ge(points))))
                    .set(rewards::points.eq(rewards::points - points))
                    .returning(rewards::username)
                    .get_result::<String>(conn)?;
            }

            diesel::insert_into(rewards_redemptions::table)
                .values(&redemption)
                .returning(rewards_redemptions::id)
                .get_result(conn)
        })
    }

    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DieselError> {
        use crate::schema::rewards_redemptions::dsl;

        if updates.is_empty() {
            return Ok(());
        }

        for update in updates {
            let target = dsl::rewards_redemptions.find(redemption_id);
            match update {
                RedemptionUpdate::Status(value) => diesel::update(target).set(dsl::status.eq(value)).execute(&mut self.connection)?,
                RedemptionUpdate::TransactionId(value) => diesel::update(target).set(dsl::transaction_id.eq(value)).execute(&mut self.connection)?,
                RedemptionUpdate::Error(value) => diesel::update(target).set(dsl::error.eq(value)).execute(&mut self.connection)?,
            };
        }

        Ok(())
    }

    fn get_redemption(&mut self, redemption_id: i32) -> Result<RewardRedemptionRow, DieselError> {
        use crate::schema::rewards_redemptions::dsl;
        dsl::rewards_redemptions
            .filter(dsl::id.eq(redemption_id))
            .select(RewardRedemptionRow::as_select())
            .first(&mut self.connection)
    }

    fn get_redemption_options(&mut self, types: &[String]) -> Result<Vec<RedemptionOptionFull>, DieselError> {
        use crate::schema::{assets, rewards_redemption_options};
        rewards_redemption_options::table
            .filter(rewards_redemption_options::redemption_type.eq_any(types))
            .left_join(assets::table.on(rewards_redemption_options::asset_id.eq(assets::id.nullable())))
            .select((RewardRedemptionOptionRow::as_select(), Option::<AssetRow>::as_select()))
            .load::<(RewardRedemptionOptionRow, Option<AssetRow>)>(&mut self.connection)
            .map(|results| results.into_iter().map(|(option, asset)| RedemptionOptionFull::new(option, asset)).collect())
    }

    fn get_redemption_option(&mut self, id: &str) -> Result<RedemptionOptionFull, DieselError> {
        use crate::schema::{assets, rewards_redemption_options};
        rewards_redemption_options::table
            .filter(rewards_redemption_options::id.eq(id))
            .left_join(assets::table.on(rewards_redemption_options::asset_id.eq(assets::id.nullable())))
            .select((RewardRedemptionOptionRow::as_select(), Option::<AssetRow>::as_select()))
            .first::<(RewardRedemptionOptionRow, Option<AssetRow>)>(&mut self.connection)
            .map(|(option, asset)| RedemptionOptionFull::new(option, asset))
    }

    fn count_redemptions_since(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DieselError> {
        use crate::schema::rewards_redemptions::dsl;
        dsl::rewards_redemptions
            .filter(dsl::username.eq(username))
            .filter(dsl::created_at.ge(since))
            .count()
            .get_result(&mut self.connection)
    }
}
