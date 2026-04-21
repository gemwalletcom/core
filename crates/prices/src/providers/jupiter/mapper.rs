use primitives::{AssetId, Chain, contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS};

use crate::AssetPriceMapping;

pub fn to_asset_price_mapping(jupiter_token_id: &str) -> AssetPriceMapping {
    if jupiter_token_id == SOLANA_WRAPPED_SOL_TOKEN_ADDRESS {
        AssetPriceMapping::new(AssetId::from_chain(Chain::Solana), Chain::Solana.as_ref().to_string())
    } else {
        AssetPriceMapping::new(AssetId::from(Chain::Solana, Some(jupiter_token_id.to_string())), jupiter_token_id.to_string())
    }
}

pub fn to_jupiter_token_id(provider_price_id: &str) -> String {
    if provider_price_id == Chain::Solana.as_ref() {
        SOLANA_WRAPPED_SOL_TOKEN_ADDRESS.to_string()
    } else {
        provider_price_id.to_string()
    }
}
