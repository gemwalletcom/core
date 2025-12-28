use crate::database::rewards::RewardsStore;
use crate::database::rewards_redemptions::{RedemptionUpdate, RewardsRedemptionsStore};
use crate::models::{NewRewardRedemptionRow, RewardRedemptionRow};
use crate::{DatabaseClient, DatabaseError};
use chrono::Utc;
use primitives::rewards::{RedemptionStatus, RewardRedemption, RewardRedemptionOption};

pub trait RewardsRedemptionsRepository {
    fn add_redemption(&mut self, username: &str, option_id: &str, device_id: i32) -> Result<RewardRedemption, DatabaseError>;
    fn get_redemption(&mut self, redemption_id: i32) -> Result<RewardRedemptionRow, DatabaseError>;
    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DatabaseError>;
    fn get_redemption_options(&mut self, types: &[String]) -> Result<Vec<RewardRedemptionOption>, DatabaseError>;
    fn get_redemption_option(&mut self, id: &str) -> Result<RewardRedemptionOption, DatabaseError>;
    fn count_redemptions_since_days(&mut self, username: &str, days: i64) -> Result<i64, DatabaseError>;
}

impl RewardsRedemptionsRepository for DatabaseClient {
    fn add_redemption(&mut self, username: &str, option_id: &str, device_id: i32) -> Result<RewardRedemption, DatabaseError> {
        let redemption_option = RewardsRedemptionsStore::get_redemption_option(self, option_id)?;
        let rewards = RewardsStore::get_rewards(self, username)?;

        if rewards.points < redemption_option.option.points {
            return Err(DatabaseError::Error("Not enough points".into()));
        }

        if redemption_option.option.remaining == Some(0) {
            return Err(DatabaseError::Error("Redemption option is no longer available".into()));
        }

        let redemption_id = RewardsRedemptionsStore::add_redemption(
            self,
            username,
            redemption_option.option.points,
            NewRewardRedemptionRow {
                username: username.to_string(),
                option_id: option_id.to_string(),
                device_id,
                status: RedemptionStatus::Pending.as_ref().to_string(),
            },
        )?;

        let option = redemption_option.as_primitive();
        let redemption_row = RewardsRedemptionsStore::get_redemption(self, redemption_id)?;
        Ok(redemption_row.as_primitive(option))
    }

    fn get_redemption(&mut self, redemption_id: i32) -> Result<RewardRedemptionRow, DatabaseError> {
        Ok(RewardsRedemptionsStore::get_redemption(self, redemption_id)?)
    }

    fn update_redemption(&mut self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DatabaseError> {
        Ok(RewardsRedemptionsStore::update_redemption(self, redemption_id, updates)?)
    }

    fn get_redemption_options(&mut self, types: &[String]) -> Result<Vec<RewardRedemptionOption>, DatabaseError> {
        let results = RewardsRedemptionsStore::get_redemption_options(self, types)?;
        Ok(results.into_iter().map(|r| r.as_primitive()).collect())
    }

    fn get_redemption_option(&mut self, id: &str) -> Result<RewardRedemptionOption, DatabaseError> {
        Ok(RewardsRedemptionsStore::get_redemption_option(self, id)?.as_primitive())
    }

    fn count_redemptions_since_days(&mut self, username: &str, days: i64) -> Result<i64, DatabaseError> {
        let since = Utc::now().naive_utc() - chrono::Duration::days(days);
        Ok(RewardsRedemptionsStore::count_redemptions_since(self, username, since)?)
    }
}
