use std::collections::HashMap;
use std::error::Error;

use api_connector::PusherClient;
use async_trait::async_trait;
use gem_tracing::info_with_fields;
use primitives::{FailedNotification, GorushNotification};
use storage::{Database, PushNotificationType};
use streamer::{NotificationsFailedPayload, NotificationsPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct NotificationsConsumer {
    pub pusher: PusherClient,
    pub stream_producer: StreamProducer,
    pub database: Database,
}

impl NotificationsConsumer {
    pub fn new(pusher: PusherClient, stream_producer: StreamProducer, database: Database) -> Self {
        Self {
            pusher,
            stream_producer,
            database,
        }
    }

    fn store_device_notifications(&self, notifications: &[GorushNotification], failures: &[FailedNotification]) {
        let device_errors: HashMap<&str, &str> = failures.iter().map(|f| (f.notification.device_id.as_str(), f.error.error.as_str())).collect();

        for notification in notifications {
            let error = device_errors.get(notification.device_id.as_str()).map(|e| e.to_string());
            let _ = self.store_device_notification(notification, error);
        }
    }

    fn store_device_notification(&self, notification: &GorushNotification, error: Option<String>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let device_row_id = self.database.client()?.devices().get_device_row_id(&notification.device_id)?;
        let notification_type = PushNotificationType::from(notification.data.notification_type.clone());
        self.database.client()?.devices_notifications().add_device_notification(device_row_id, notification_type, error)?;
        Ok(())
    }
}

#[async_trait]
impl MessageConsumer<NotificationsPayload, usize> for NotificationsConsumer {
    async fn should_process(&self, _payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: NotificationsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let result = self.pusher.push_notifications(payload.notifications).await?;
        let counts = result.response.counts as usize;
        let success = &result.response.success;
        let logs = &result.response.logs;

        info_with_fields!("gorush response", counts = counts, success = success.as_str(), logs = format!("{:?}", logs));

        let failures = result.failures();

        self.store_device_notifications(&result.notifications, &failures);

        if !failures.is_empty() {
            info_with_fields!(
                "push failures",
                count = failures.len(),
                failures = format!("{:?}", failures.iter().map(|f| (&f.notification.device_id, &f.error.error)).collect::<Vec<_>>())
            );
            self.stream_producer.publish_notifications_failed(NotificationsFailedPayload::new(failures)).await?;
        }

        Ok(counts)
    }
}
