use serde::Deserialize;

use super::assets::Asset;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub external_transaction_id: Option<String>,
    pub status: String,
    pub base_currency_amount: Option<f64>,
    pub quote_currency_amount: Option<f64>,
    pub base_currency: Asset,
    pub quote_currency: Option<Asset>,
    pub crypto_transaction_id: Option<String>,
    pub network_fee_amount: Option<f64>,
    pub extra_fee_amount: Option<f64>,
    pub fee_amount: Option<f64>,
}
