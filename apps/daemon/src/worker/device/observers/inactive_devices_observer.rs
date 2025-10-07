use cacher::{CacherClient, INACTIVE_DEVICE_OBSERVER, INACTIVE_DEVICE_OBSERVER_TIMEOUT};
use localizer::LanguageLocalizer;
use primitives::{Asset, Chain, GorushNotification, PushNotification};
use std::error::Error;
use storage::database::DatabaseClient;
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

pub struct InactiveDevicesObserver {
    database: DatabaseClient,
    cacher: CacherClient,
    stream_producer: StreamProducer,
}

impl InactiveDevicesObserver {
    pub fn new(database_url: &str, cacher: CacherClient, stream_producer: StreamProducer) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
            cacher,
            stream_producer,
        }
    }

    pub async fn observe(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        // 7 days to 14 days
        let devices = self.database.devices().devices_inactive_days(10, 14, Some(true))?;
        for device in &devices {
            let subscriptions = self.database.subscriptions().get_subscriptions_by_device_id(&device.id, None)?;
            if subscriptions.is_empty() {
                continue;
            }
            if !self
                .cacher
                .can_process_now(&format!("{}:{}", INACTIVE_DEVICE_OBSERVER, device.id), INACTIVE_DEVICE_OBSERVER_TIMEOUT)
                .await?
            {
                continue;
            }
            let language_localizer = LanguageLocalizer::new_with_language(&device.locale);
            let asset = Asset::from_chain(Chain::Bitcoin);
            let (title, description) = language_localizer.notification_onboarding_buy_asset(&asset.name);
            let notification = GorushNotification::from_device(device.clone(), title, description, PushNotification::new_buy_asset(asset.id));
            let payload = NotificationsPayload::new(vec![notification]);
            self.stream_producer.publish_notifications_observers(payload).await?;
        }

        Ok(devices.len())
    }
}
