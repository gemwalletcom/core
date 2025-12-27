use gem_rewards::{RewardsRedemptionError, redeem_points};
use primitives::ConfigKey;
use primitives::rewards::RedemptionResult;
use storage::Database;
use streamer::{StreamProducer, StreamProducerQueue};

pub struct RewardsRedemptionClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl RewardsRedemptionClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn redeem(&mut self, address: &str, id: &str, device_id: i32) -> Result<RedemptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let rewards = client.rewards().get_reward_by_address(address)?;

        if !rewards.is_enabled {
            return Err(RewardsRedemptionError::NotEligible("Not eligible for rewards".to_string()).into());
        }

        let username = rewards.code.ok_or(RewardsRedemptionError::NoUsername)?;

        self.check_redemption_limits(&username)?;

        let response = redeem_points(&mut client, &username, id, device_id)?;
        self.stream_producer
            .publish_rewards_redemption(streamer::RewardsRedemptionPayload::new(response.redemption_id))
            .await?;

        Ok(response.result)
    }

    fn check_redemption_limits(&mut self, username: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.database.client()?;

        let daily_limit = client.config().get_config_i64(ConfigKey::RedemptionPerUserDaily)?;
        let daily_count = client.rewards_redemptions().count_redemptions_since_days(username, 1)?;
        if daily_count >= daily_limit {
            return Err(RewardsRedemptionError::DailyLimitReached.into());
        }

        let weekly_limit = client.config().get_config_i64(ConfigKey::RedemptionPerUserWeekly)?;
        let weekly_count = client.rewards_redemptions().count_redemptions_since_days(username, 7)?;
        if weekly_count >= weekly_limit {
            return Err(RewardsRedemptionError::WeeklyLimitReached.into());
        }

        Ok(())
    }
}
