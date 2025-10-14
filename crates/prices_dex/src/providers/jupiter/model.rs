use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Price {
    pub price: f64,
    pub price_change_24h: f64,
}

pub type JupiterPriceResponse = HashMap<String, TokenPriceData>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPriceData {
    #[serde(rename = "usdPrice")]
    pub usd_price: f64,
    #[serde(rename = "priceChange24h")]
    #[serde(default)]
    pub price_change_24h: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopTokensResponse {
    pub tokens: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifiedToken {
    pub id: String,
}

pub type VerifiedTokensResponse = Vec<VerifiedToken>;
