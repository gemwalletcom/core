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
pub struct WebhookEncryptedData {
    pub data: String,
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
    pub status: String,
    pub fiat_currency: String,
    pub is_buy_or_sell: String,
    pub fiat_amount: f64,
    pub network: Option<String>,
    pub crypto_currency: String,
    pub transaction_hash: Option<String>,
    pub wallet_address: String,
    pub conversion_price_data: Option<WebhookConversionPriceData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookConversionPriceData {
    pub internal_fees: Vec<InternalFee>,
}

impl WebhookConversionPriceData {
    pub fn fee(&self, id: &str) -> Option<f64> {
        self.internal_fees.iter().find(|fee| fee.id == id).map(|fee| fee.value)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InternalFee {
    pub id: String,
    pub value: f64,
}
