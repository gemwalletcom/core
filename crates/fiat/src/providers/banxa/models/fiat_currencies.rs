use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatCurrency {
    pub id: String,
    pub description: String,
    pub symbol: String,
    pub supported_payment_methods: Vec<FiatPaymentMethod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatPaymentMethod {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub minimum: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub maximum: f64,
}
