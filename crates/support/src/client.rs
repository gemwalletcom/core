use crate::ChatwootWebhookPayload;
use localizer::LanguageLocalizer;
use primitives::{Device, GorushNotification, PushNotification, PushNotificationTypes, push_notification::PushNotificationSupport};
use std::error::Error;
use storage::{Database, DatabaseClient};
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

pub struct SupportClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl SupportClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn handle_message_created(&self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let device = client.get_support_device(support_device_id)?.as_primitive();

        let notifications_count = if let Some(notification) = Self::build_notification(&device, payload) {
            self.stream_producer.publish_notifications_support(NotificationsPayload::new(vec![notification])).await?;
            1
        } else {
            0
        };

        Self::update_unread(&mut client, support_device_id, payload)?;
        Ok(notifications_count)
    }

    pub fn handle_conversation_updated(&self, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        Self::update_unread(&mut client, support_device_id, payload)?;
        Ok(())
    }

    fn update_unread(client: &mut DatabaseClient, support_device_id: &str, payload: &ChatwootWebhookPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(unread) = payload.get_unread() {
            client.support().support_update_unread(support_device_id, unread)?;
        }
        Ok(())
    }

    fn build_notification(device: &Device, payload: &ChatwootWebhookPayload) -> Option<GorushNotification> {
        if !payload.is_outgoing_message() {
            return None;
        }

        let title = LanguageLocalizer::new_with_language(&device.locale).notification_support_new_message_title();
        let message = payload.content.clone().unwrap_or_default();
        let data = PushNotification {
            notification_type: PushNotificationTypes::Support,
            data: serde_json::to_value(PushNotificationSupport {}).ok(),
        };

        Some(GorushNotification::from_device(device.clone(), title, message, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_notification_message_created() {
        let payload: ChatwootWebhookPayload =
            serde_json::from_str(include_str!("../tests/testdata/chatwoot_message_created.json")).unwrap();

        let notification = SupportClient::build_notification(&Device::mock(), &payload);

        assert!(notification.is_some());
        assert_eq!(notification.unwrap().message, "from agent");
    }

    #[test]
    fn test_build_notification_conversation_updated() {
        let payload: ChatwootWebhookPayload =
            serde_json::from_str(include_str!("../tests/testdata/chatwoot_conversation_updated.json")).unwrap();

        let notification = SupportClient::build_notification(&Device::mock(), &payload);

        assert!(notification.is_none());
    }
}
