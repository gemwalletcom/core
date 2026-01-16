use gem_rewards::{RewardsRedemptionError, redeem_points};
use primitives::rewards::{RedemptionResult, Rewards};
use primitives::{ConfigKey, NaiveDateTimeExt, now};
use storage::{ConfigCacher, Database, RewardsRedemptionsRepository, RewardsRepository, WalletsRepository};
use streamer::{StreamProducer, StreamProducerQueue};

pub struct RewardsRedemptionClient {
    database: Database,
    config: ConfigCacher,
    stream_producer: StreamProducer,
}

impl RewardsRedemptionClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            database,
            config,
            stream_producer,
        }
    }

    pub async fn redeem(&self, wallet_identifier: &str, id: &str, device_id: i32) -> Result<RedemptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let wallet = self.database.wallets()?.get_wallet(wallet_identifier)?;
        let rewards = self.database.rewards()?.get_reward_by_wallet_id(wallet.id)?;

        if !rewards.status.is_enabled() {
            return Err(RewardsRedemptionError::NotEligible("Not eligible for rewards".to_string()).into());
        }

        let username = rewards.code.clone().ok_or(RewardsRedemptionError::NoUsername)?;

        self.check_redemption_limits(&username, &rewards)?;

        let response = redeem_points(&mut self.database.client()?, &username, id, device_id)?;
        self.stream_producer
            .publish_rewards_redemption(streamer::RewardsRedemptionPayload::new(response.redemption_id))
            .await?;

        Ok(response.result)
    }

    fn check_redemption_limits(&self, username: &str, rewards: &Rewards) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let current = now();

        if rewards.created_at > current.ago(self.config.get_duration(ConfigKey::RedemptionMinAccountAge)?) {
            return Err(RewardsRedemptionError::AccountTooNew.into());
        }

        let cooldown_since = current.ago(self.config.get_duration(ConfigKey::RedemptionCooldownAfterReferral)?);
        if self.database.rewards()?.count_referrals_since(username, cooldown_since)? > 0 {
            return Err(RewardsRedemptionError::CooldownNotElapsed.into());
        }

        let daily_limit = self.config.get_i64(ConfigKey::RedemptionPerUserDaily)?;
        let daily_count = self.database.rewards_redemptions()?.count_redemptions_since_days(username, 1)?;
        if daily_count >= daily_limit {
            return Err(RewardsRedemptionError::DailyLimitReached.into());
        }

        let weekly_limit = self.config.get_i64(ConfigKey::RedemptionPerUserWeekly)?;
        let weekly_count = self.database.rewards_redemptions()?.count_redemptions_since_days(username, 7)?;
        if weekly_count >= weekly_limit {
            return Err(RewardsRedemptionError::WeeklyLimitReached.into());
        }

        Ok(())
    }
}
