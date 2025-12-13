use gem_auth::{AuthClient, verify_auth_signature};
use primitives::{AuthMessage, NaiveDateTimeExt, Rewards, RewardsEventItem, RewardsReferralRequest};
use std::sync::Arc;
use storage::Database;
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

const REFERRAL_ELIGIBILITY_DAYS: i64 = 7;

pub struct RewardsClient {
    database: Database,
    stream_producer: StreamProducer,
    auth_client: Arc<AuthClient>,
}

impl RewardsClient {
    pub fn new(database: Database, stream_producer: StreamProducer, auth_client: Arc<AuthClient>) -> Self {
        Self {
            database,
            stream_producer,
            auth_client,
        }
    }

    pub fn get_rewards(&mut self, address: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_by_address(address)?)
    }

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardsEventItem>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub async fn create_referral(&mut self, request: &RewardsReferralRequest) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request).await?;
        let (rewards, event_id) = self.database.client()?.rewards().create_reward(&address, &request.code)?;
        self.publish_event(event_id).await?;
        Ok(rewards)
    }

    pub async fn use_referral_code(&mut self, request: &RewardsReferralRequest) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request).await?;
        let device = self.database.client()?.get_device(&request.auth.device_id)?;

        if !device.created_at.is_within_days(REFERRAL_ELIGIBILITY_DAYS) {
            return Err("Device not eligible for referral".into());
        }

        if let Some(date) = self.database.client()?.rewards().get_first_subscription_date(vec![address.clone()])? {
            if !date.is_within_days(REFERRAL_ELIGIBILITY_DAYS) {
                return Err("Address not eligible for referral".into());
            }
        }

        let event_ids = self.database.client()?.rewards().use_referral_code(&address, &request.code, device.id)?;
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

    async fn verify_request(&self, request: &RewardsReferralRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let auth_nonce = self.auth_client.get_auth_nonce(&request.auth.device_id, &request.auth.nonce).await?;

        self.database.client()?.get_device(&request.auth.device_id)?;

        let auth_message = AuthMessage {
            chain: request.auth.chain,
            address: request.auth.address.clone(),
            auth_nonce,
        };

        if !verify_auth_signature(&auth_message, &request.auth.signature) {
            return Err("Authentication failed".into());
        }

        self.auth_client.invalidate_nonce(&request.auth.device_id, &request.auth.nonce).await?;

        Ok(request.auth.address.clone())
    }
}
