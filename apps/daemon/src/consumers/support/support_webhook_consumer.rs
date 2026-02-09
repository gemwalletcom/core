use std::error::Error;

use async_trait::async_trait;
use gem_tracing::{error_with_fields, info_with_fields};
use settings::Settings;
use storage::Database;
use streamer::consumer::MessageConsumer;
use streamer::{StreamProducer, StreamProducerConfig, SupportWebhookPayload};

use primitives::Device;
use support::{ChatwootWebhookPayload, EVENT_CONVERSATION_STATUS_CHANGED, EVENT_CONVERSATION_UPDATED, EVENT_MESSAGE_CREATED, SupportClient};

pub struct SupportWebhookConsumer {
    support_client: SupportClient,
}

impl SupportWebhookConsumer {
    pub async fn new(settings: &Settings) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database = Database::new(&settings.postgres.url, settings.postgres.pool);
        let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
        let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
        let stream_producer = StreamProducer::new(&rabbitmq_config, "daemon_support_producer").await?;
        Ok(Self {
            support_client: SupportClient::new(database, stream_producer),
        })
    }

    async fn process_webhook(&self, device: &Device, webhook: &ChatwootWebhookPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
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
                error_with_fields!("Support webhook parsing failed", &e, payload = payload.data.to_string());
                return Ok(true);
            }
        };

        let Some(device_id) = webhook.get_device_id() else {
            info_with_fields!("Support webhook missing device_id", event = webhook.event);
            return Ok(true);
        };

        let Some(device) = self.support_client.get_device(&device_id)? else {
            info_with_fields!("Support webhook device not found", device_id = device_id);
            return Ok(true);
        };

        match self.process_webhook(&device, &webhook).await {
            Ok(notifications) => {
                info_with_fields!("Support webhook processed", device_id = device_id, event = webhook.event, notifications = notifications);
                Ok(true)
            }
            Err(error) => {
                error_with_fields!("Support webhook failed", &*error, device_id = device_id, payload = payload.data.to_string());
                Err(error)
            }
        }
    }
}
