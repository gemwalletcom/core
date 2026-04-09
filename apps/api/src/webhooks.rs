use gem_tracing::info_with_fields;
use primitives::{TransactionId, WebhookKind};
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::{State, post, serde::json::Json, tokio::sync::Mutex};
use std::str::FromStr;
use storage::Database;
use storage::database::webhooks::WebhooksStore;
use streamer::{QueueName, StreamProducer, SupportWebhookPayload};

use crate::devices::FiatQuotesClient;
use crate::responders::ApiError;

pub struct WebhooksClient {
    stream_producer: StreamProducer,
}

impl WebhooksClient {
    pub fn new(stream_producer: StreamProducer) -> Self {
        Self { stream_producer }
    }

    pub async fn process_support_webhook(&self, webhook_data: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = SupportWebhookPayload::new(webhook_data);
        self.stream_producer.publish(QueueName::SupportWebhooks, &payload).await?;
        Ok(())
    }

    pub async fn process_broadcast_webhook(&self, payload: TransactionId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let transaction_id = payload.to_string();
        info_with_fields!("received broadcast webhook", transaction_id = transaction_id.as_str());
        self.stream_producer.publish(QueueName::StorePendingTransactions, &payload).await?;
        info_with_fields!("published broadcast webhook", transaction_id = transaction_id.as_str());
        Ok(())
    }
}

pub struct WebhookKindParam(WebhookKind);

impl<'r> FromParam<'r> for WebhookKindParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        WebhookKind::from_str(param).map(Self).map_err(|_| param)
    }
}

fn authorize_webhook(database: &State<Database>, kind: WebhookKind, sender: &str, secret: &str) -> Result<(), ApiError> {
    let enabled = database
        .client()
        .and_then(|mut c| WebhooksStore::get_webhook_endpoint(&mut c, kind, sender, secret).map_err(Into::into))
        .map_err(|_| ApiError::InternalServerError("Failed to load webhook endpoint".to_string()))?
        .ok_or_else(|| ApiError::NotFound("Webhook endpoint not found".to_string()))?;

    if !enabled {
        return Err(ApiError::NotFound("Webhook endpoint not found".to_string()));
    }

    Ok(())
}

#[post("/webhooks/<kind>/<sender>/<secret>", data = "<webhook_data>")]
pub async fn create_webhook(
    kind: WebhookKindParam,
    sender: &str,
    secret: &str,
    database: &State<Database>,
    webhook_data: Json<serde_json::Value>,
    fiat_quotes_client: &State<Mutex<FiatQuotesClient>>,
    webhooks_client: &State<Mutex<WebhooksClient>>,
) -> Result<Status, ApiError> {
    authorize_webhook(database, kind.0, sender, secret)?;

    let webhook_data = webhook_data.0;
    match kind.0 {
        WebhookKind::Transactions => {
            let payload: TransactionId = serde_json::from_value(webhook_data)?;
            webhooks_client.lock().await.process_broadcast_webhook(payload).await?;
        }
        WebhookKind::Support | WebhookKind::SupportBot => {
            webhooks_client.lock().await.process_support_webhook(webhook_data).await?;
        }
        WebhookKind::Fiat => {
            fiat_quotes_client.lock().await.process_and_publish_webhook(sender, webhook_data).await?;
        }
    }
    Ok(Status::Ok)
}
