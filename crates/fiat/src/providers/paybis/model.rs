use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub amount: String,
    pub direction_change: String,
    pub is_received_amount: bool,
    pub currency_code_from: String,
    pub currency_code_to: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisQuote {
    pub currency_code_to: String,
    pub payment_methods: Vec<PaymentMethod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod {
    pub amount_to: AmountInfo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountInfo {
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisAssetsResponse {
    pub meta: MetaData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetaData {
    pub currencies: Vec<Currency>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub code: String,
    pub blockchain_name: Option<String>,
}

impl Currency {
    pub fn is_crypto(&self) -> bool {
        self.blockchain_name.is_some()
    }

    pub fn unsupported_countries(&self) -> HashMap<String, Vec<String>> {
        HashMap::new()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisWebhook {
    pub id: String,
    pub status: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_currency: String,
    pub wallet_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub country: Option<String>,
    pub network_fee: Option<f64>,
    pub service_fee: Option<f64>,
    pub partner_fee: Option<f64>,
}
