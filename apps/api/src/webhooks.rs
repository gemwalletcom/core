use crate::devices::FiatQuotesClient;
use crate::responders::{ApiError, ApiResponse};
use rocket::{State, post, serde::json::Json, tokio::sync::Mutex};
use streamer::{FiatWebhook, QueueName, StreamProducer, SupportWebhookPayload};

pub struct WebhooksClient {
    stream_producer: StreamProducer,
}

impl WebhooksClient {
    pub fn new(stream_producer: StreamProducer) -> Self {
        Self { stream_producer }
    }

    pub async fn process_support_webhook(&self, webhook_data: serde_json::Value) -> Result<SupportWebhookPayload, Box<dyn std::error::Error + Send + Sync>> {
        let payload = SupportWebhookPayload::new(webhook_data.clone());
        self.stream_producer.publish(QueueName::SupportWebhooks, &payload).await?;
        Ok(payload)
    }
}

#[post("/webhooks/support", data = "<webhook_data>")]
pub async fn create_support_webhook(webhook_data: Json<serde_json::Value>, webhooks_client: &State<Mutex<WebhooksClient>>) -> Result<ApiResponse<SupportWebhookPayload>, ApiError> {
    Ok(webhooks_client.lock().await.process_support_webhook(webhook_data.0).await?.into())
}

#[post("/webhooks/support/bot", data = "<webhook_data>")]
pub async fn create_support_bot_webhook(
    webhook_data: Json<serde_json::Value>,
    webhooks_client: &State<Mutex<WebhooksClient>>,
) -> Result<ApiResponse<SupportWebhookPayload>, ApiError> {
    Ok(webhooks_client.lock().await.process_support_webhook(webhook_data.0).await?.into())
}

#[post("/fiat/webhooks/<provider>", data = "<webhook_data>")]
pub async fn create_fiat_webhook(provider: &str, webhook_data: Json<serde_json::Value>, client: &State<Mutex<FiatQuotesClient>>) -> Result<ApiResponse<FiatWebhook>, ApiError> {
    Ok(client.lock().await.process_and_publish_webhook(provider, webhook_data.0).await?.payload.into())
}
