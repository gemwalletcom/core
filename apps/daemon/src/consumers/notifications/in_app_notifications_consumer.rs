use std::error::Error;

use async_trait::async_trait;
use localizer::LanguageLocalizer;
use primitives::{Device, GorushNotification, NotificationType, PushNotification, PushNotificationReward, PushNotificationTypes, RewardEventType};
use storage::{Database, NewNotificationRow, NotificationType as StorageNotificationType, NotificationsRepository, WalletsRepository};
use streamer::{InAppNotificationPayload, NotificationsPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct InAppNotificationsConsumer {
    database: Database,
    stream_producer: StreamProducer,
}

impl InAppNotificationsConsumer {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    fn create_push_notification(&self, device: &Device, notification_type: NotificationType, wallet_id: i32, points: Option<i32>) -> GorushNotification {
        let localizer = LanguageLocalizer::new_with_language(&device.locale);
        let (title, message) = notification_content(&localizer, notification_type, points.unwrap_or(0));
        let data = PushNotification {
            notification_type: PushNotificationTypes::Rewards,
            data: serde_json::to_value(PushNotificationReward { wallet_id: wallet_id.to_string() }).ok(),
        };
        GorushNotification::from_device(device.clone(), title, message, data)
    }
}

#[async_trait]
impl MessageConsumer<InAppNotificationPayload, usize> for InAppNotificationsConsumer {
    async fn should_process(&self, _payload: InAppNotificationPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: InAppNotificationPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let notification = NewNotificationRow {
            wallet_id: payload.wallet_id,
            notification_type: StorageNotificationType::from(payload.notification_type),
            metadata: payload.metadata.clone(),
        };
        self.database.notifications()?.create_notifications(vec![notification])?;

        let points = payload.metadata.as_ref().and_then(|m| m.get("points")).and_then(|p| p.as_i64()).map(|p| p as i32);

        let devices: Vec<Device> = self
            .database
            .wallets()?
            .get_devices_by_wallet_id(payload.wallet_id)?
            .into_iter()
            .map(|d| d.as_primitive())
            .collect();

        let notifications: Vec<GorushNotification> = devices
            .iter()
            .filter(|d| d.can_receive_push_notification())
            .map(|device| self.create_push_notification(device, payload.notification_type, payload.wallet_id, points))
            .collect();

        let count = notifications.len();
        self.stream_producer.publish_notifications_rewards(NotificationsPayload::new(notifications)).await?;

        Ok(count)
    }
}

fn notification_content(localizer: &LanguageLocalizer, notification_type: NotificationType, points: i32) -> (String, String) {
    match notification_type {
        NotificationType::RewardsCreateUsername => (
            localizer.notification_reward_title(RewardEventType::CreateUsername.points()),
            localizer.notification_reward_create_username_description(),
        ),
        NotificationType::RewardsInvite => (
            localizer.notification_reward_title(RewardEventType::InviteNew.points()),
            localizer.notification_reward_invite_description(),
        ),
        NotificationType::ReferralJoined => (
            localizer.notification_reward_title(RewardEventType::Joined.points()),
            localizer.notification_reward_joined_description(),
        ),
        NotificationType::RewardsEnabled => (localizer.notification_reward_title(0), localizer.notification_rewards_enabled_description()),
        NotificationType::RewardsCodeDisabled => (localizer.notification_rewards_disabled_title(), localizer.notification_rewards_disabled_description()),
        NotificationType::RewardsRedeemed => (localizer.notification_reward_redeemed_title(), localizer.notification_reward_redeemed_description(points)),
    }
}
