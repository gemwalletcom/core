use primitives::{Rewards, RewardsEventItem, RewardsReferralRequest};
use storage::Database;
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

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

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardsEventItem>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub async fn create_referral(&mut self, request: &RewardsReferralRequest) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request)?;
        let (rewards, event_id) = self.database.client()?.rewards().create_reward(&address, &request.code)?;
        self.publish_event(event_id).await?;
        Ok(rewards)
    }

    pub async fn use_referral_code(&mut self, request: &RewardsReferralRequest) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request)?;
        let device = self.database.client()?.get_device(&request.device_id)?;
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

    fn verify_request(&self, request: &RewardsReferralRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !referral::verify_siwe_signature(&request.message, &request.signature, &request.address) {
            return Err("Invalid signature".into());
        }

        let message = referral::parse_siwe_message(&request.message).ok_or("Invalid message format")?;
        Ok(message.address)
    }
}
