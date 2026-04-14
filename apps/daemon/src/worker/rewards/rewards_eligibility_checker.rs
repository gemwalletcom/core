use std::error::Error;

use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{NotificationType, RewardStatus};
use storage::{Database, RewardsFilter, RewardsRepository};
use streamer::{InAppNotificationPayload, RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

pub struct RewardsEligibilityChecker {
    database: Database,
    stream_producer: StreamProducer,
}

impl RewardsEligibilityChecker {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn check(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let usernames = self
            .database
            .rewards()?
            .get_rewards_by_filter(vec![RewardsFilter::Statuses(vec![RewardStatus::Pending, RewardStatus::Unverified])])?
            .into_iter()
            .map(|reward| reward.username)
            .collect::<Vec<_>>();
        let mut promoted = 0;

        for username in usernames {
            let result = match self.evaluate_and_promote(&username).await {
                Ok(result) => result,
                Err(error) => {
                    error_with_fields!("rewards eligibility check failed", &*error, username = username);
                    continue;
                }
            };

            if result {
                promoted += 1;
            }
        }

        if promoted > 0 {
            info_with_fields!("rewards eligibility promoted users", count = promoted);
        }

        Ok(promoted)
    }

    async fn evaluate_and_promote(&self, username: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let Some((wallet_id, reward_event_ids)) = self.database.rewards()?.promote_pending_reward(username)? else {
            return Ok(false);
        };

        self.publish_promotion(wallet_id, reward_event_ids).await?;
        Ok(true)
    }

    async fn publish_promotion(&self, wallet_id: i32, reward_event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer
            .publish_in_app_notifications(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsEnabled, None)])
            .await?;

        self.stream_producer
            .publish_rewards_events(reward_event_ids.into_iter().map(RewardsNotificationPayload::new).collect())
            .await?;

        Ok(())
    }
}
