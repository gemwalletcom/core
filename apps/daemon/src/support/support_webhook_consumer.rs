use std::error::Error;

use async_trait::async_trait;
use gem_tracing::{error_with_context, info_with_context};
use settings::Settings;
use storage::DatabaseClient;
use streamer::consumer::MessageConsumer;
use streamer::{StreamProducer, SupportWebhookPayload};

use super::model::ChatwootWebhookPayload;
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
        let webhook_payload = serde_json::from_value::<ChatwootWebhookPayload>(payload.data.clone())?;
        let device_id = webhook_payload.get_device_id().unwrap_or_default();

        match self.support_client.process_webhook(&webhook_payload).await {
            Ok(_) => {
                info_with_context(
                    "Support webhook processed",
                    &[
                        ("device_id", device_id.as_str()),
                        ("event", webhook_payload.event.as_str()),
                        ("message_type", webhook_payload.message_type.as_str()),
                    ],
                );
            }
            Err(e) => {
                error_with_context("Failed to process support webhook", e.as_ref(), &[("payload", &payload.data.to_string())]);
                return Err(e);
            }
        }

        Ok(true)
    }
}
