use std::error::Error;

use async_trait::async_trait;
use gem_tracing::{error_with_fields, info_with_fields};
use streamer::SupportWebhookPayload;
use streamer::consumer::MessageConsumer;

use primitives::Device;
use support::{ChatwootWebhookPayload, EVENT_CONVERSATION_STATUS_CHANGED, EVENT_CONVERSATION_UPDATED, EVENT_MESSAGE_CREATED, SupportBotClient, SupportClient};

pub struct SupportWebhookConsumer {
    support_client: SupportClient,
    bot_client: SupportBotClient,
}

impl SupportWebhookConsumer {
    pub fn new(support_client: SupportClient, bot_client: SupportBotClient) -> Self {
        Self { support_client, bot_client }
    }

    async fn process_notification(&self, device: &Device, webhook: &ChatwootWebhookPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        match webhook.event.as_str() {
            EVENT_MESSAGE_CREATED => self.support_client.handle_message_created(device, webhook).await,
            EVENT_CONVERSATION_UPDATED | EVENT_CONVERSATION_STATUS_CHANGED => self.support_client.handle_conversation_updated(webhook).map(|_| 0),
            _ => Ok(0),
        }
    }
}

#[async_trait]
impl MessageConsumer<SupportWebhookPayload, bool> for SupportWebhookConsumer {
    async fn should_process(&self, _payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let webhook = match serde_json::from_value::<ChatwootWebhookPayload>(payload.data.clone()) {
            Ok(w) => w,
            Err(e) => {
                error_with_fields!("support webhook parsing failed", &e, payload = payload.data.to_string());
                return Ok(true);
            }
        };

        if webhook.event == EVENT_MESSAGE_CREATED && webhook.is_incoming_message() {
            return match self.bot_client.process_incoming(&webhook).await {
                Ok(result) => Ok(result),
                Err(error) => {
                    error_with_fields!("bot webhook processing failed", &*error, payload = payload.data.to_string());
                    Err(error)
                }
            };
        }

        let Some(device_id) = webhook.get_device_id() else {
            info_with_fields!("support webhook missing device_id", event = webhook.event);
            return Ok(true);
        };

        let Some(device) = self.support_client.get_device(&device_id)? else {
            info_with_fields!("support webhook device not found", device_id = device_id);
            return Ok(true);
        };

        match self.process_notification(&device, &webhook).await {
            Ok(notifications) => {
                info_with_fields!("support webhook processed", device_id = device_id, event = webhook.event, notifications = notifications);
                Ok(true)
            }
            Err(error) => {
                error_with_fields!("support webhook failed", &*error, device_id = device_id, payload = payload.data.to_string());
                Err(error)
            }
        }
    }
}
