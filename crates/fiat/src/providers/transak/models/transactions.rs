use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransakOrderResponse {
    pub id: String,
    pub status: String,
    pub fiat_currency: String,
    pub is_buy_or_sell: String,
    pub fiat_amount: f64,
    pub crypto_currency: String,
    pub network: String,
    pub transaction_hash: Option<String>,
    pub wallet_address: Option<String>,
    pub country_code: Option<String>,
}