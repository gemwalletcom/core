use std::error::Error;

use async_trait::async_trait;
use gem_tracing::{error_with_fields, info_with_fields};
use settings::Settings;
use storage::DatabaseClient;
use streamer::consumer::MessageConsumer;
use streamer::{StreamProducer, SupportWebhookPayload};

use super::model::{ChatwootWebhookPayload, EVENT_CONVERSATION_UPDATED, EVENT_MESSAGE_CREATED};
use super::support_client::SupportClient;

pub struct SupportWebhookConsumer {
    support_client: SupportClient,
}

impl SupportWebhookConsumer {
    pub async fn new(settings: &Settings) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database = Box::new(DatabaseClient::new(&settings.postgres.url));
        let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "daemon_support_producer").await?;
        let support_client = SupportClient::new(database, stream_producer);
        Ok(Self { support_client })
    }
}

#[async_trait]
impl MessageConsumer<SupportWebhookPayload, bool> for SupportWebhookConsumer {
    async fn should_process(&mut self, _payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&mut self, payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let webhook_payload = match serde_json::from_value::<ChatwootWebhookPayload>(payload.data.clone()) {
            Ok(payload) => payload,
            Err(e) => {
                error_with_fields!("Support webhook parsing failed", &e, payload = payload.data.to_string());
                return Err(e.into());
            }
        };

        let support_device_id = match webhook_payload.get_support_device_id() {
            Some(support_device_id) => support_device_id,
            None => {
                info_with_fields!("Support webhook missing support_device_id, skipping", event = webhook_payload.event);
                return Ok(true);
            }
        };

        let result = match webhook_payload.event.as_str() {
            EVENT_MESSAGE_CREATED => self.support_client.handle_message_created(&support_device_id, &webhook_payload).await,
            EVENT_CONVERSATION_UPDATED => self.support_client.handle_conversation_updated(&support_device_id, &webhook_payload),
            _ => {
                info_with_fields!(
                    "Support webhook event skipped",
                    support_device_id = support_device_id,
                    event = webhook_payload.event
                );
                return Ok(true);
            }
        };
        match result {
            Ok(_) => {
                info_with_fields!(
                    "Support webhook processed",
                    support_device_id = support_device_id,
                    event = webhook_payload.event
                );
                Ok(true)
            }
            Err(error) => {
                error_with_fields!("Support webhook failed", &*error, payload = payload.data.to_string());
                Err(error)
            }
        }
    }
}
