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
    pub id: String,
    pub chain: Option<Chain>,
    pub symbol: String,
    pub token_id: Option<String>,
    pub network: Option<String>,
    pub enabled: bool,
}

pub type FiatMappingMap = HashMap<String, FiatMapping>;

// used to filter out fiat tokens that have specific token ids for native coins
pub fn filter_token_id(token_id: Option<String>) -> Option<String> {
    token_id.filter(|contract_address| {
        ![
            "0x0000000000000000000000000000000000001010", // matic
            "0x0000000000000000000000000000000000000000",
            "0x471ece3750da237f93b8e339c536989b8978a438", // celo
        ]
        .contains(&contract_address.as_str())
    })
}
