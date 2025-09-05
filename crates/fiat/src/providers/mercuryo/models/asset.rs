use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Currencies {
    pub config: Config,
    pub fiat_payment_methods: HashMap<String, FiatPaymentMethod>,
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
pub struct FiatPaymentMethod {
    pub payment_methods: Vec<PaymentMethod>,
    pub limits: Limits,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PaymentMethod {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Limits {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub min: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub max: f64,
}
