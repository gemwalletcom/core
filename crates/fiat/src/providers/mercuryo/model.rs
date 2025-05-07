use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_f64_from_str, deserialize_option_f64_from_str};

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Quote {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
    pub currency: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteQuery {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub network: String,
    pub widget_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteSellQuery {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub quote_type: String,
    pub amount: f64,
    pub network: String,
    pub widget_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Currencies {
    pub config: Config,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub crypto_currencies: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub currency: String,
    pub network: String,
    pub contract: String,
}

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
