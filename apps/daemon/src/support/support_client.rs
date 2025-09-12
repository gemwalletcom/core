use super::model::{ChatwootWebhookPayload, EVENT_MESSAGE_CREATED, MESSAGE_TYPE_OUTGOING};
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

    pub async fn process_webhook(&mut self, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        if payload.event != EVENT_MESSAGE_CREATED && payload.message_type != MESSAGE_TYPE_OUTGOING {
            return Ok(());
        }
        let device_id = payload.get_device_id().ok_or("Device ID not found")?;
        let device = self.database.devices().get_device(&device_id)?;
        self.send_support_push_notification(device, payload).await?;

        Ok(())
    }

    async fn send_support_push_notification(
        &mut self,
        device: primitives::Device,
        payload: &ChatwootWebhookPayload,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let language_localizer = LanguageLocalizer::new_with_language(&device.locale);
        let title = language_localizer.notification_support_new_message_title();
        let message = payload.content.clone().unwrap_or_default();

        let data = PushNotification {
            notification_type: PushNotificationTypes::Support,
            data: serde_json::to_value(PushNotificationSupport {
                conversation_id: payload.conversation.id,
            })
            .ok(),
        };
        let notification = GorushNotification::from_device(device, title, message, data);
        let notifications_payload = NotificationsPayload::new(vec![notification]);
        self.stream_producer.publish_notifications_support(notifications_payload).await?;

        Ok(())
    }
}
