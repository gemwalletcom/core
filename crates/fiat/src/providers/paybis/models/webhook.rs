use serde::Deserialize;

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
