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
