use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    pub data: WebhookData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookData {
    pub id: String,
    pub merchant_transaction_id: Option<String>,
    pub status: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub tx: Option<WebhookTransaction>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookTransaction {
    pub id: Option<String>,
}
