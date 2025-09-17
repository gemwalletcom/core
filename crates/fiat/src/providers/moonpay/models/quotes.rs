use serde::Deserialize;

use super::assets::Currency;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayBuyQuote {
    pub quote_currency_amount: f64,
    pub quote_currency_code: String,
    pub quote_currency: Currency,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPaySellQuote {
    pub base_currency_amount: f64,
    pub base_currency_code: String,
    pub quote_currency_amount: f64,
    pub base_currency: Currency,
}
