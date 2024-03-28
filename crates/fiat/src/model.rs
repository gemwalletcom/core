use primitives::Chain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct FiatRequestMap {
    pub crypto_currency: String,
    pub network: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatRates {
    pub rates: Vec<storage::models::FiatRate>,
}

// mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatMapping {
    pub symbol: String,
    pub network: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FiatProviderAsset {
    pub chain: Chain,
    pub symbol: String,
    pub token_id: Option<String>,
    pub network: Option<String>,
    pub enabled: bool,
}

pub type FiatMappingMap = HashMap<String, FiatMapping>;
