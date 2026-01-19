use std::error::Error;

use async_trait::async_trait;
use primitives::{NotificationRewardsMetadata, NotificationType, RewardEventType};
use storage::{Database, RewardsRepository};
use streamer::{InAppNotificationPayload, RewardsNotificationPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct RewardsConsumer {
    database: Database,
    stream_producer: StreamProducer,
}

#[async_trait]
impl MessageConsumer<RewardsNotificationPayload, usize> for RewardsConsumer {
    async fn should_process(&self, _payload: RewardsNotificationPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: RewardsNotificationPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let event = self.database.rewards()?.get_reward_event(payload.event_id)?;
        let notifications = self.create_in_app_notification_payloads(&event)?;
        let count = notifications.len();
        self.stream_producer.publish_in_app_notifications(notifications).await?;
        Ok(count)
    }
}

impl RewardsConsumer {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    fn create_in_app_notification_payloads(&self, event: &primitives::RewardEvent) -> Result<Vec<InAppNotificationPayload>, Box<dyn Error + Send + Sync>> {
        let metadata = NotificationRewardsMetadata {
            username: event.username.clone(),
            points: Some(event.points),
        };
        let metadata_value = serde_json::to_value(metadata).ok();

        match event.event {
            RewardEventType::CreateUsername => {
                let wallet_id = self.database.rewards()?.get_wallet_id_by_username(&event.username)?;
                Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsCreateUsername, metadata_value)])
            }
            RewardEventType::InvitePending | RewardEventType::InviteNew | RewardEventType::InviteExisting => {
                let wallet_id = self.database.rewards()?.get_wallet_id_by_username(&event.username)?;
                Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsInvite, metadata_value)])
            }
            RewardEventType::Joined => {
                let Some(referrer) = self.database.rewards()?.get_referrer_username(&event.username)? else {
                    return Ok(vec![]);
                };
                let wallet_id = self.database.rewards()?.get_wallet_id_by_username(&referrer)?;
                Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::ReferralJoined, metadata_value)])
            }
            RewardEventType::Disabled => {
                let wallet_id = self.database.rewards()?.get_wallet_id_by_username(&event.username)?;
                Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsCodeDisabled, metadata_value)])
            }
            RewardEventType::Redeemed => {
                let wallet_id = self.database.rewards()?.get_wallet_id_by_username(&event.username)?;
                Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsRedeemed, metadata_value)])
            }
        }
    }
}
