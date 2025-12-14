use crate::auth::VerifiedAuth;
use primitives::{NaiveDateTimeExt, RewardEvent, RewardEventType, Rewards};
use storage::Database;
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

const REFERRAL_ELIGIBILITY_DAYS: i64 = 7;

pub struct RewardsClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl RewardsClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub fn get_rewards(&mut self, address: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_by_address(address)?)
    }

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardEvent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub async fn create_referral(&mut self, address: &str, code: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        let (rewards, event_id) = self.database.client()?.rewards().create_reward(address, code)?;
        self.publish_event(event_id).await?;
        Ok(rewards)
    }

    pub async fn use_referral_code(&mut self, auth: &VerifiedAuth, code: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        let event_ids = self
            .database
            .client()?
            .rewards()
            .use_referral_code(&auth.address, code, device.id, invite_event)?;
        self.publish_events(event_ids).await?;
        Ok(())
    }

    async fn publish_event(&self, event_id: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = vec![RewardsNotificationPayload::new(event_id)];
        self.stream_producer.publish_rewards_events(payload).await?;
        Ok(())
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload: Vec<RewardsNotificationPayload> = event_ids.into_iter().map(RewardsNotificationPayload::new).collect();
        self.stream_producer.publish_rewards_events(payload).await?;
        Ok(())
    }
}
