use crate::ChatwootWebhookPayload;
use localizer::LanguageLocalizer;
use primitives::{Device, GorushNotification, PushNotification, PushNotificationTypes, push_notification::PushNotificationSupport};
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

    pub async fn handle_message_created(&self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        let device = self.database.client()?.get_support_device(support_device_id)?.as_primitive();

        if let Some(notification) = Self::build_notification(device, payload) {
            let notifications_payload = NotificationsPayload::new(vec![notification]);
            self.stream_producer.publish_notifications_support(notifications_payload).await?;
        }

        if let Some(unread) = payload.get_unread() {
            self.database.client()?.support().support_update_unread(support_device_id, unread)?;
        }

        Ok(())
    }

    pub fn handle_conversation_updated(&self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(unread) = payload.get_unread() {
            let mut client = self.database.client()?;
            client.support().support_update_unread(support_device_id, unread)?;
        }

        Ok(())
    }

    pub fn build_notification(device: Device, payload: &ChatwootWebhookPayload) -> Option<GorushNotification> {
        if !payload.is_outgoing_message() {
            return None;
        }

        let language_localizer = LanguageLocalizer::new_with_language(&device.locale);
        let title = language_localizer.notification_support_new_message_title();
        let message = payload.content.clone().unwrap_or_default();

        let data = PushNotification {
            notification_type: PushNotificationTypes::Support,
            data: serde_json::to_value(PushNotificationSupport {}).ok(),
        };

        Some(GorushNotification::from_device(device, title, message, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_notification_message_created() {
        let payload: ChatwootWebhookPayload =
            serde_json::from_str(include_str!("../tests/testdata/chatwoot_message_created.json")).unwrap();
        let device = Device::mock();

        let notification = SupportClient::build_notification(device, &payload);

        assert!(notification.is_some());
        let notification = notification.unwrap();
        assert_eq!(notification.message, "from agent");
    }

    #[test]
    fn test_build_notification_conversation_updated() {
        let payload: ChatwootWebhookPayload =
            serde_json::from_str(include_str!("../tests/testdata/chatwoot_conversation_updated.json")).unwrap();
        let device = Device::mock();

        let notification = SupportClient::build_notification(device, &payload);

        assert!(notification.is_none());
    }
}
