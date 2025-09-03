use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

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
