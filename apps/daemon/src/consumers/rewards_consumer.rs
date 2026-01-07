use std::error::Error;

use async_trait::async_trait;
use localizer::LanguageLocalizer;
use primitives::{Device, GorushNotification, PushNotification, PushNotificationReward, PushNotificationTypes, RewardEventType};
use storage::{Database, RewardsRepository};
use streamer::{NotificationsPayload, RewardsNotificationPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct RewardsConsumer {
    database: Database,
    stream_producer: StreamProducer,
}

impl RewardsConsumer {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }
}

#[async_trait]
impl MessageConsumer<RewardsNotificationPayload, usize> for RewardsConsumer {
    async fn should_process(&self, _payload: RewardsNotificationPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: RewardsNotificationPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let event = self.database.rewards()?.get_reward_event(payload.event_id)?;
        let devices = self.database.rewards()?.get_reward_event_devices(payload.event_id)?;

        let notifications: Vec<GorushNotification> = devices
            .into_iter()
            .filter(|d| d.can_receive_push_notification())
            .map(|device| create_notification(device, event.event))
            .collect();

        if notifications.is_empty() {
            return Ok(0);
        }

        let count = notifications.len();
        self.stream_producer
            .publish_notifications_rewards(NotificationsPayload::new(notifications))
            .await?;
        Ok(count)
    }
}

fn create_notification(device: Device, event: RewardEventType) -> GorushNotification {
    let localizer = LanguageLocalizer::new_with_language(&device.locale);
    let (title, message) = reward_notification_content(&localizer, event);
    let data = PushNotification {
        notification_type: PushNotificationTypes::Rewards,
        data: serde_json::to_value(PushNotificationReward {}).ok(),
    };
    GorushNotification::from_device(device, title, message, data)
}

fn reward_notification_content(localizer: &LanguageLocalizer, event: RewardEventType) -> (String, String) {
    match event {
        RewardEventType::CreateUsername => (
            localizer.notification_reward_title(event.points()),
            localizer.notification_reward_create_username_description(),
        ),
        RewardEventType::InviteNew | RewardEventType::InviteExisting => (
            localizer.notification_reward_title(event.points()),
            localizer.notification_reward_invite_description(),
        ),
        RewardEventType::Joined => (
            localizer.notification_reward_title(event.points()),
            localizer.notification_reward_joined_description(),
        ),
        RewardEventType::Disabled => (
            localizer.notification_rewards_disabled_title(),
            localizer.notification_rewards_disabled_description(),
        ),
    }
}
