use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Quote {
    pub amount: String,
    pub currency: String,
    pub fiat_amount: String,
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
    pub fiat_amount: String,
    pub fee: Option<String>,
    pub partner_fee: Option<String>,
    pub fiat_currency: String,
    pub currency: String,
    pub merchant_transaction_id: Option<String>,
    pub tx: Option<Transaction>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    pub id: Option<String>,
    pub address: Option<String>,
}
