use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub crypto_amount: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
}