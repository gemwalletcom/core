use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayBuyQuote {
    pub quote_currency_amount: f64,
    pub quote_currency_code: String,
    pub quote_currency: Currency,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub decimals: u32,
    pub not_allowed_countries: Vec<String>,
    #[serde(rename = "notAllowedUSStates")]
    pub not_allowed_us_states: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPaySellQuote {
    pub base_currency_amount: f64,
    pub base_currency_code: String,
    pub quote_currency_amount: f64,
    pub base_currency: Currency,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayIpAddress {
    pub alpha2: String,
    pub state: String,
    pub is_buy_allowed: bool,
    pub is_sell_allowed: bool,
    pub is_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub alpha2: String,
    pub is_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub code: String,
    pub metadata: Option<CurrencyMetadata>,
    pub is_suspended: Option<bool>,
    //#[serde(rename = "notAllowedUSStates")]
    //pub not_allowed_us_states: Option<Vec<String>>,
    #[serde(rename = "notAllowedCountries")]
    pub not_allowed_countries: Option<Vec<String>>,
}

impl Asset {
    pub fn unsupported_countries(&self) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();

        for country in &self.not_allowed_countries.clone().unwrap_or_default() {
            map.insert(country.clone(), vec![]);
        }
        map
    }
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
