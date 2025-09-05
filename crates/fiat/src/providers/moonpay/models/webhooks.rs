use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookOrderId {
    pub id: String,
}