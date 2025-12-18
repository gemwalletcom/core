use gem_rewards::{RewardsError, redeem_points};
use primitives::rewards::RedemptionResult;
use storage::{Database, RewardsRepository};
use streamer::{StreamProducer, StreamProducerQueue};

pub struct RewardsRedemptionClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl RewardsRedemptionClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn redeem(&mut self, address: &str, id: &str) -> Result<RedemptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let rewards = client.get_reward_by_address(address)?;

        if !rewards.is_enabled {
            return Err(RewardsError::Redemption("Not eligible for rewards".to_string()).into());
        }

        let username = rewards.code.ok_or(RewardsError::Username("No username found for address".to_string()))?;

        let response = redeem_points(&mut client, &username, id)?;
        self.stream_producer
            .publish_rewards_redemption(streamer::RewardsRedemptionPayload::new(response.redemption_id))
            .await?;

        Ok(response.result)
    }
}
