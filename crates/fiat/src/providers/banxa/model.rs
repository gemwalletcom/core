use std::collections::HashMap;

use serde::Deserialize;
use serde_serializers::{deserialize_f64_from_str, deserialize_option_f64_from_str};

pub const ORDER_TYPE_BUY: &str = "buy";
pub const ORDER_TYPE_SELL: &str = "sell";

#[derive(Debug, Deserialize, Clone)]
pub struct Coin {
    pub id: String,
    pub blockchain: String,
    pub address: String,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct Coins {
    pub coins: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub id: String,
    pub blockchains: Vec<Blockchain>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Blockchain {
    pub id: String,
    pub address: Option<String>,
    pub unsupported_countries: UnsupportedCountries,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum UnsupportedCountries {
    Map(HashMap<String, Vec<String>>),
    Empty(Vec<()>),
}

impl UnsupportedCountries {
    pub fn list_map(self) -> HashMap<String, Vec<String>> {
        match self {
            UnsupportedCountries::Map(map) => map,
            UnsupportedCountries::Empty(_) => HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub crypto_amount: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod {
    pub id: String,
    pub supported_fiats: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub id: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: String,
    pub status: String,
    pub crypto: Coin,
    pub fiat: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub crypto_amount: f64,
    pub wallet_address: String,
    pub tx_hash: Option<String>,
    #[serde(deserialize_with = "deserialize_option_f64_from_str")]
    pub processing_fee: Option<f64>,
    #[serde(deserialize_with = "deserialize_option_f64_from_str")]
    pub network_fee: Option<f64>,
    pub order_type: String,
    pub country: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    pub order_id: String,
}
