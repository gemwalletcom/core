use serde::Deserialize;
use serde_serializers::{deserialize_f64_from_str, deserialize_option_f64_from_str};

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    pub data: WebhookData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookData {
    pub id: String,
    pub status: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    #[serde(default, deserialize_with = "deserialize_option_f64_from_str")]
    pub fee: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64_from_str")]
    pub partner_fee: Option<f64>,
    pub fiat_currency: String,
    pub currency: String,
    pub merchant_transaction_id: Option<String>,
    pub tx: Option<Transaction>,
    #[serde(rename = "type")]
    pub transacton_type: String,
    pub user: Option<User>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub country_code: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    pub id: Option<String>,
    pub address: Option<String>,
}
