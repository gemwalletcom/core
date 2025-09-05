use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub coin_id: String,
    pub unique_id: String,
    pub symbol: String,
    pub network: AssetNetwork,
    pub address: Option<String>,
    pub is_allowed: bool,
    pub kyc_countries_not_supported: Vec<String>,
}

impl Asset {
    pub fn unsupported_countries(&self) -> HashMap<String, Vec<String>> {
        self.kyc_countries_not_supported.clone().into_iter().map(|country| (country, vec![])).collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetNetwork {
    pub name: String,
}