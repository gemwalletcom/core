use gem_auth::verify_device_token;
use gem_tracing::info_with_fields;
use primitives::TransactionId;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State, post, serde::json::Json, tokio::sync::Mutex};
use streamer::{FiatWebhook, QueueName, StreamProducer, SupportWebhookPayload};

use crate::devices::FiatQuotesClient;
use crate::devices::auth_config::AuthConfig;
use crate::responders::{ApiError, ApiResponse, cache_error};

pub struct WebhooksClient {
    stream_producer: StreamProducer,
}

impl WebhooksClient {
    pub fn new(stream_producer: StreamProducer) -> Self {
        Self { stream_producer }
    }

    pub async fn process_support_webhook(&self, webhook_data: serde_json::Value) -> Result<SupportWebhookPayload, Box<dyn std::error::Error + Send + Sync>> {
        let payload = SupportWebhookPayload::new(webhook_data);
        self.stream_producer.publish(QueueName::SupportWebhooks, &payload).await?;
        Ok(payload)
    }

    pub async fn process_broadcast_webhook(&self, payload: TransactionId) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let transaction_id = payload.to_string();
        info_with_fields!("received broadcast webhook", transaction_id = transaction_id.as_str());
        self.stream_producer.publish(QueueName::StorePendingTransactions, &payload).await?;
        info_with_fields!("published broadcast webhook", transaction_id = transaction_id.as_str());
        Ok(Some(payload.hash))
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

pub struct WebhookAuthorized;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebhookAuthorized {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let Success(config) = req.guard::<&rocket::State<AuthConfig>>().await else {
            cache_error(req, "Auth config not available");
            return Error((Status::InternalServerError, "Auth config not available".to_string()));
        };

        let Some(auth_value) = req.headers().get_one("Authorization") else {
            cache_error(req, "Missing Authorization header");
            return Error((Status::Unauthorized, "Missing Authorization header".to_string()));
        };

        let Some(token) = auth_value.strip_prefix("Bearer ") else {
            cache_error(req, "Invalid authorization format");
            return Error((Status::Unauthorized, "Invalid authorization format".to_string()));
        };

        if verify_device_token(token, &config.jwt.secret).is_err() {
            cache_error(req, "Invalid webhook token");
            return Error((Status::Unauthorized, "Invalid webhook token".to_string()));
        }

        Success(WebhookAuthorized)
    }
}

#[post("/webhooks/transactions", data = "<payload>")]
pub async fn create_broadcast_webhook(
    _auth: WebhookAuthorized,
    payload: Json<TransactionId>,
    webhooks_client: &State<Mutex<WebhooksClient>>,
) -> Result<ApiResponse<Option<String>>, ApiError> {
    Ok(webhooks_client.lock().await.process_broadcast_webhook(payload.0).await?.into())
}

#[post("/fiat/webhooks/<provider>", data = "<webhook_data>")]
pub async fn create_fiat_webhook(provider: &str, webhook_data: Json<serde_json::Value>, client: &State<Mutex<FiatQuotesClient>>) -> Result<ApiResponse<FiatWebhook>, ApiError> {
    Ok(client.lock().await.process_and_publish_webhook(provider, webhook_data.0).await?.payload.into())
}

#[cfg(test)]
mod tests {
    use gem_auth::create_device_token;
    use rocket::http::{ContentType, Header, Status};
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket, post, routes};
    use std::time::Duration;

    use super::WebhookAuthorized;
    use crate::devices::auth_config::{AuthConfig, JwtConfig};

    const TEST_SECRET: &str = "test_jwt_secret";

    #[post("/webhooks/transactions")]
    async fn protected(_auth: WebhookAuthorized) -> &'static str {
        "ok"
    }

    fn auth_config(secret: &str) -> AuthConfig {
        AuthConfig::new(
            true,
            Duration::from_secs(30),
            JwtConfig {
                secret: secret.to_string(),
                expiry: Duration::from_secs(60),
            },
        )
    }

    fn rocket(config: AuthConfig) -> Rocket<Build> {
        rocket::build().manage(config).mount("/", routes![protected])
    }

    fn valid_token() -> Header<'static> {
        let (token, _) = create_device_token("dynode", TEST_SECRET, Duration::from_secs(60)).unwrap();
        Header::new("Authorization", format!("Bearer {token}"))
    }

    #[rocket::async_test]
    async fn test_webhook_auth() {
        let client = Client::tracked(rocket(auth_config(TEST_SECRET))).await.unwrap();

        let response = client.post("/webhooks/transactions").header(ContentType::JSON).dispatch().await;
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client.post("/webhooks/transactions").header(valid_token()).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}
