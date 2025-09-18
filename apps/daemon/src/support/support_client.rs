use super::model::ChatwootWebhookPayload;
use localizer::LanguageLocalizer;
use primitives::{push_notification::PushNotificationSupport, GorushNotification, PushNotification, PushNotificationTypes};
use std::error::Error;
use storage::DatabaseClient;
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

pub struct SupportClient {
    database: Box<DatabaseClient>,
    stream_producer: StreamProducer,
}

impl SupportClient {
    pub fn new(database: Box<DatabaseClient>, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn handle_message_created(&mut self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        let device = self.database.get_support_device(support_device_id)?.as_primitive();

        if let Some(unread) = payload.get_unread() {
            self.database.support().support_update_unread(support_device_id, unread)?;
        }

        let language_localizer = LanguageLocalizer::new_with_language(&device.locale);
        let title = language_localizer.notification_support_new_message_title();
        let message = payload.content.clone().unwrap_or_default();

        let data = PushNotification {
            notification_type: PushNotificationTypes::Support,
            data: serde_json::to_value(PushNotificationSupport {}).ok(),
        };
        let notification = GorushNotification::from_device(device, title, message, data);
        let notifications_payload = NotificationsPayload::new(vec![notification]);
        self.stream_producer.publish_notifications_support(notifications_payload).await?;

        Ok(())
    }

    pub fn handle_conversation_updated(&mut self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(unread) = payload.get_unread() {
            self.database.support().support_update_unread(support_device_id, unread)?;
        }

        Ok(())
    }
}
