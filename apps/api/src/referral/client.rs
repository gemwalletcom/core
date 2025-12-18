use gem_rewards::{IpSecurityClient, RewardsError};
use primitives::{ConfigKey, NaiveDateTimeExt, RewardEvent, RewardEventType, Rewards};
use storage::Database;
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

use crate::auth::VerifiedAuth;

const REFERRAL_ELIGIBILITY_DAYS: i64 = 7;

pub struct RewardsClient {
    database: Database,
    stream_producer: StreamProducer,
    ip_security_client: IpSecurityClient,
}

impl RewardsClient {
    pub fn new(database: Database, stream_producer: StreamProducer, ip_security_client: IpSecurityClient) -> Self {
        Self {
            database,
            stream_producer,
            ip_security_client,
        }
    }

    pub fn get_rewards(&mut self, address: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_by_address(address)?)
    }

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardEvent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub async fn create_referral(&mut self, address: &str, code: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        let (rewards, event_id) = self.database.client()?.rewards().create_reward(address, code)?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    pub async fn use_referral_code(&mut self, auth: &VerifiedAuth, code: &str, ip_address: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let device = self.database.client()?.get_device(&auth.device_id)?;
        let first_subscription_date = self.database.client()?.rewards().get_first_subscription_date(vec![auth.address.clone()])?;

        let is_new_device = device.created_at.is_within_days(REFERRAL_ELIGIBILITY_DAYS);
        let is_new_subscription = first_subscription_date
            .map(|date| date.is_within_days(REFERRAL_ELIGIBILITY_DAYS))
            .unwrap_or(true);

        let is_new_user = is_new_device && is_new_subscription;

        let invite_event = if is_new_user {
            RewardEventType::InviteNew
        } else {
            RewardEventType::InviteExisting
        };

        if !self.ip_security_client.is_eligible(ip_address).await? {
            return Err(RewardsError::Referral("Not eligible for referral rewards".to_string()).into());
        }

        let daily_limit = self.database.client()?.config().get_config_value_i64(ConfigKey::ReferralPerIpDaily)?;
        let weekly_limit = self.database.client()?.config().get_config_value_i64(ConfigKey::ReferralPerIpWeekly)?;

        if !self.ip_security_client.can_use_referral(ip_address, daily_limit, weekly_limit).await? {
            return Err(RewardsError::Referral("Not eligible for referral rewards".to_string()).into());
        }

        let event_ids = self
            .database
            .client()?
            .rewards()
            .use_referral_code(&auth.address, code, device.id, invite_event)?;

        self.ip_security_client.record_referral_usage(ip_address).await?;
        self.publish_events(event_ids).await?;
        Ok(())
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for event_id in event_ids {
            let payload = RewardsNotificationPayload::new(event_id);
            self.stream_producer.publish_rewards_events(vec![payload]).await?;
        }
        Ok(())
    }
}
