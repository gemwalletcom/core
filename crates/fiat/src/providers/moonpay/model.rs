use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayQuote {
    pub quote_currency_amount: f64,
    pub quote_currency_code: String,
    pub quote_currency: QuooteCurrency,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuooteCurrency {
    pub not_allowed_countries: Vec<String>,
    #[serde(rename = "notAllowedUSStates")]
    pub not_allowed_us_states: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayIpAddress {
    pub alpha2: String,
    pub state: String,
    pub is_buy_allowed: bool,
    pub is_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub code: String,
    pub metadata: Option<CurrencyMetadata>,
    pub is_suspended: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyMetadata {
    pub contract_address: Option<String>,
    pub network_code: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data<T> {
    pub data: T,
    #[serde(rename = "type")]
    pub transaction_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub id: String,
    pub status: String,
    pub base_currency_amount: f64,
    pub base_currency: FiatCurrency,
    pub currency: Asset,
    pub wallet_address: Option<String>,
    pub crypto_transaction_id: Option<String>,
    pub network_fee_amount: Option<f64>,
    pub extra_fee_amount: Option<f64>,
    pub fee_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatCurrency {
    pub code: String,
}
