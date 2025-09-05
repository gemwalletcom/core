use serde::Deserialize;

use super::assets::Asset;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub status: String,
    pub base_currency_amount: Option<f64>,
    pub quote_currency_amount: Option<f64>,
    pub base_currency: Asset,
    pub currency: Option<Asset>,
    pub quote_currency: Option<Asset>,
    pub wallet_address: Option<String>,
    pub refund_wallet_address: Option<String>,
    pub crypto_transaction_id: Option<String>,
    pub network_fee_amount: Option<f64>,
    pub extra_fee_amount: Option<f64>,
    pub fee_amount: Option<f64>,
    pub country: Option<String>,
    pub failure_reason: Option<String>,
}