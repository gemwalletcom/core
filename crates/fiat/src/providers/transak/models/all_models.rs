use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransakQuote {
    pub quote_id: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_currency: String,
    pub crypto_amount: f64,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub response: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub coin_id: String,
    pub unique_id: String,
    pub symbol: String,
    pub network: AssetNetwork,
    pub address: Option<String>,
    pub is_allowed: bool,
    pub kyc_countries_not_supported: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub alpha2: String,
    pub is_allowed: bool,
}

impl Asset {
    pub fn unsupported_countries(&self) -> HashMap<String, Vec<String>> {
        self.kyc_countries_not_supported.clone().into_iter().map(|country| (country, vec![])).collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetNetwork {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    pub webhook_data: WebhookData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookData {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransakOrderResponse {
    pub id: String,
    pub status: String,
    pub fiat_currency: String,
    pub is_buy_or_sell: String,
    pub fiat_amount: f64,
    pub crypto_currency: String,
    pub network: String,
    pub transaction_hash: Option<String>,
    pub wallet_address: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Data<T> {
    pub data: T,
}
