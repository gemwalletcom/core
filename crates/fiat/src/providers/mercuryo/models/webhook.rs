use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    pub data: WebhookData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookData {
    pub id: String,
    pub merchant_transaction_id: Option<String>,
}
