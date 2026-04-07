use std::error::Error;

use async_trait::async_trait;
use localizer::LanguageLocalizer;
use number_formatter::{ValueFormatter, ValueStyle};
use primitives::{
    Device, GorushNotification, JsonDecode, NotificationRewardsRedeemMetadata, NotificationType, PushNotification, PushNotificationReward, PushNotificationTypes, RewardEventType,
};
use storage::{AssetsRepository, Database, NewNotificationRow, NotificationType as StorageNotificationType, NotificationsRepository, WalletsRepository};
use streamer::{InAppNotificationPayload, NotificationsPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct InAppNotificationsConsumer {
    database: Database,
    stream_producer: StreamProducer,
}

impl InAppNotificationsConsumer {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    fn create_push_notification(
        &self,
        device: &Device,
        notification_type: NotificationType,
        wallet_id: i32,
        points: i32,
        reward_value: Option<&str>,
    ) -> Option<GorushNotification> {
        let localizer = LanguageLocalizer::new_with_language(device.locale.as_str());
        let (title, message) = notification_content(&localizer, notification_type, points, reward_value);
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
        let redeem: Option<NotificationRewardsRedeemMetadata> = payload.metadata.decode();
        let reward_value = redeem.as_ref().and_then(|m| {
            let asset_id = payload.asset_id.as_ref()?;
            let asset = self.database.assets().ok()?.get_asset(&asset_id.to_string()).ok()?;
            ValueFormatter::format_with_symbol(ValueStyle::Auto, &m.value, asset.decimals, &asset.symbol).ok()
        });
        let points = redeem.as_ref().map(|m| m.points).unwrap_or(0);

        let notification = NewNotificationRow {
            wallet_id: payload.wallet_id,
            asset_id: payload.asset_id.map(Into::into),
            notification_type: StorageNotificationType::from(payload.notification_type),
            metadata: payload.metadata.clone(),
        };
        self.database.notifications()?.create_notifications(vec![notification])?;

        let devices: Vec<Device> = self
            .database
            .wallets()?
            .get_devices_by_wallet_id(payload.wallet_id)?
            .into_iter()
            .map(|d| d.as_primitive())
            .collect();

        let notifications: Vec<GorushNotification> = devices
            .iter()
            .filter_map(|device| self.create_push_notification(device, payload.notification_type, payload.wallet_id, points, reward_value.as_deref()))
            .collect();

        let count = notifications.len();
        self.stream_producer.publish_notifications_rewards(NotificationsPayload::new(notifications)).await?;

        Ok(count)
    }
}

fn notification_content(localizer: &LanguageLocalizer, notification_type: NotificationType, points: i32, reward_value: Option<&str>) -> (String, String) {
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
        NotificationType::RewardsRedeemed => (
            localizer.notification_reward_redeemed_title(),
            localizer.notification_reward_redeemed_description(points, reward_value),
        ),
    }
}
