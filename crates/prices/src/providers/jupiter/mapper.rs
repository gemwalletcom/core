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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jupiter_price_id_mapping() {
        let mapping = to_asset_price_mapping(SOLANA_WRAPPED_SOL_TOKEN_ADDRESS);
        assert_eq!(mapping.asset_id, AssetId::from_chain(Chain::Solana));
        assert_eq!(mapping.provider_price_id, Chain::Solana.as_ref());
        assert_eq!(to_jupiter_token_id(&mapping.provider_price_id), SOLANA_WRAPPED_SOL_TOKEN_ADDRESS);

        let token = "BPxxfRCXkUVhig4HS1Lh7kZqV6SPJhzfEk4x6fVBjPCy";
        let mapping = to_asset_price_mapping(token);
        assert_eq!(mapping.asset_id, AssetId::from_token(Chain::Solana, token));
        assert_eq!(mapping.provider_price_id, token);
        assert_eq!(to_jupiter_token_id(&mapping.provider_price_id), token);
    }
}
