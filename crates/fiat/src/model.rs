use chain_primitives::format_token_id;
use primitives::{AssetId, Chain, CosmosDenom};
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
    pub unsupported_countries: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FiatProviderAsset {
    pub id: String,
    pub chain: Option<Chain>,
    pub symbol: String,
    pub token_id: Option<String>,
    pub network: Option<String>,
    pub enabled: bool,
    pub unsupported_countries: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct FiatProviderCountry {
    pub alpha2: String,
    pub is_allowed: bool,
}

impl FiatProviderAsset {
    pub fn asset_id(&self) -> Option<AssetId> {
        match self.clone().chain {
            Some(chain) => match &self.token_id {
                Some(token_id) => format_token_id(chain, token_id.to_string()).map(|formatted_token_id| AssetId::from(chain, Some(formatted_token_id))),
                None => Some(chain.as_asset_id()),
            },
            None => None,
        }
    }
}

pub type FiatMappingMap = HashMap<String, FiatMapping>;

// used to filter out fiat tokens that have specific token ids for native coins
pub fn filter_token_id(chain: Option<Chain>, token_id: Option<String>) -> Option<String> {
    let token_id = token_id.filter(|contract_address| {
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
    });
    if let Some(chain) = chain {
        if let Some(token_id) = token_id {
            return format_token_id(chain, token_id);
        }
    }
    token_id
}
