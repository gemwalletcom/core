use super::model::{ChatwootWebhookPayload, MESSAGE_TYPE_INCOMING};
use localizer::LanguageLocalizer;
use primitives::{GorushNotification, PushNotification, PushNotificationTypes, push_notification::PushNotificationSupport};
use std::error::Error;
use storage::Database;
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

pub struct SupportClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl SupportClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn handle_message_created(&mut self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let device = client.get_support_device(support_device_id)?.as_primitive();

        if let Some(unread) = payload.get_unread() {
            self.database.client()?.support().support_update_unread(support_device_id, unread)?;
        }

        if let Some(message_type) = payload.message_type.clone()
            && message_type == MESSAGE_TYPE_INCOMING
        {
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
        }

        Ok(())
    }

    pub fn handle_conversation_updated(&mut self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(unread) = payload.get_unread() {
            self.database.client()?.support().support_update_unread(support_device_id, unread)?;
        }

        Ok(())
    }
}
