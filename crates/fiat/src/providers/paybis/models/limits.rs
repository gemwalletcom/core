use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaybisData<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodWithLimits {
    pub name: String,
    pub pairs: Vec<CurrencyPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyPair {
    pub from: String,
    pub to: Vec<CurrencyLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyLimit {
    pub currency: String,
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
    #[serde(rename = "minAmount", deserialize_with = "deserialize_f64_from_str")]
    pub min_amount: f64,
    #[serde(rename = "maxAmount", deserialize_with = "deserialize_f64_from_str")]
    pub max_amount: f64,
}
