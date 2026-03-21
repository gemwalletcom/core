use chain_primitives::format_token_id;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{
    Asset, AssetId, Chain, CosmosDenom, FiatAssetSymbol, FiatProviderName,
    asset_constants::WORLD_WETH_TOKEN_ID,
    contract_constants::{EVM_ZERO_ADDRESS, SOLANA_SYSTEM_PROGRAM_ID},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FiatMapping {
    pub asset: Asset,
    pub asset_symbol: FiatAssetSymbol,
    pub unsupported_countries: HashMap<String, Vec<String>>,
    pub buy_limits: Vec<FiatAssetLimits>,
    pub sell_limits: Vec<FiatAssetLimits>,
}

#[derive(Debug, Clone)]
pub struct FiatProviderAsset {
    pub id: String,
    pub provider: FiatProviderName,
    pub chain: Option<Chain>,
    pub symbol: String,
    pub token_id: Option<String>,
    pub network: Option<String>,
    pub enabled: bool,
    pub is_buy_enabled: bool,
    pub is_sell_enabled: bool,
    pub unsupported_countries: Option<HashMap<String, Vec<String>>>,
    pub buy_limits: Vec<FiatAssetLimits>,
    pub sell_limits: Vec<FiatAssetLimits>,
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

impl FiatMapping {
    pub fn get_network(network: Option<String>) -> Result<String, crate::error::FiatQuoteError> {
        network.ok_or_else(|| crate::error::FiatQuoteError::InvalidRequest("Missing network".to_string()))
    }
}

pub type FiatMappingMap = HashMap<String, FiatMapping>;

// used to filter out fiat tokens that have specific token ids for native coins
pub fn filter_token_id(chain: Option<Chain>, token_id: Option<String>) -> Option<String> {
    let token_id = token_id.filter(|contract_address| {
        ![
            "0x0000000000000000000000000000000000001010", // matic
            EVM_ZERO_ADDRESS,
            "0x471ece3750da237f93b8e339c536989b8978a438", // celo
            WORLD_WETH_TOKEN_ID,                          // worldchain
            CosmosDenom::Uosmo.as_ref(),                  // osmosis
            CosmosDenom::Usei.as_ref(),                   // sei
            CosmosDenom::Inj.as_ref(),                    // osmosis
            CosmosDenom::Uusdc.as_ref(),                  // noble
            CosmosDenom::Uatom.as_ref(),                  // atom
            CosmosDenom::Rune.as_ref(),                   // rune
            CosmosDenom::Utia.as_ref(),                   // celestia
            SOLANA_SYSTEM_PROGRAM_ID,
        ]
        .contains(&contract_address.as_str())
    });
    if let Some(chain) = chain
        && let Some(token_id) = token_id
    {
        return format_token_id(chain, token_id);
    }
    token_id
}
