use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize)]
struct TransakPayload<'a> {
    ip_address: &'a str,
    fiat_currency: &'a str,
    crypto_currency: &'a str,
    is_buy_or_sell: &'a str,
    fiat_amount: f64,
    network: &'a str,
    partner_api_key: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct TransakResponse<T> {
    pub response: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub coin_id: String,
    pub symbol: String,
    pub network: AssetNetwork,
    pub address: Option<String>,
    pub is_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetNetwork {
    pub name: String,
}
