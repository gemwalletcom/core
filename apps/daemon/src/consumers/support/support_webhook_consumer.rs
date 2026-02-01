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
        let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
        let stream_producer = StreamProducer::new(&rabbitmq_config, "daemon_support_producer").await?;
        Ok(Self {
            support_client: SupportClient::new(database, stream_producer),
        })
    }

    async fn process_webhook(&self, device: &Device, support_device_id: &str, webhook: &ChatwootWebhookPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        match webhook.event.as_str() {
            EVENT_MESSAGE_CREATED => self.support_client.handle_message_created(device, support_device_id, webhook).await,
            EVENT_CONVERSATION_UPDATED | EVENT_CONVERSATION_STATUS_CHANGED => self.support_client.handle_conversation_updated(support_device_id, webhook).map(|_| 0),
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

        let Some(support_device_id) = webhook.get_support_device_id() else {
            info_with_fields!("Support webhook missing support_device_id", event = webhook.event);
            return Ok(true);
        };

        let Some(device) = self.support_client.get_device(&support_device_id)? else {
            info_with_fields!("Support webhook device not found", support_device_id = support_device_id);
            return Ok(true);
        };

        match self.process_webhook(&device, &support_device_id, &webhook).await {
            Ok(notifications) => {
                info_with_fields!(
                    "Support webhook processed",
                    support_device_id = support_device_id,
                    event = webhook.event,
                    notifications = notifications
                );
                Ok(true)
            }
            Err(error) => {
                error_with_fields!("Support webhook failed", &*error, support_device_id = support_device_id, payload = payload.data.to_string());
                Err(error)
            }
        }
    }
}
