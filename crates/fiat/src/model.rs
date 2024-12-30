use primitives::{Chain, CosmosDenom};
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
            CosmosDenom::Uosmo.as_ref(),                  // osmosis
            CosmosDenom::Usei.as_ref(),                   // sei
            CosmosDenom::Inj.as_ref(),                    // osmosis
            CosmosDenom::Uusdc.as_ref(),                  // noble
            CosmosDenom::Uatom.as_ref(),                  // atom
            CosmosDenom::Rune.as_ref(),                   // rune
            CosmosDenom::Utia.as_ref(),                   // celestia
            "bip122:1a91e3dace36e2be3bf030a65679fe82",    // banxa::DOGE
            "bip122:12a765e31ffd4059bada1e25190f6e98",    // banxa::LTC
        ]
        .contains(&contract_address.as_str())
    })
}
