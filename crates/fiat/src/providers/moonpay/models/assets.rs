use serde::Deserialize;
use std::collections::HashMap;

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
pub struct Asset {
    pub code: String,
    pub metadata: Option<CurrencyMetadata>,
    pub is_suspended: Option<bool>,
    #[serde(rename = "notAllowedCountries")]
    pub not_allowed_countries: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub currency_type: FiatCurrencyType,
    pub min_buy_amount: Option<f64>,
    pub max_buy_amount: Option<f64>,
    pub min_sell_amount: Option<f64>,
    pub max_sell_amount: Option<f64>,
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FiatCurrencyType {
    Fiat,
    Crypto,
}
