use primitives::{AssetId, AssetMarket, Chain, Price, PriceProvider, contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS};

use crate::{AssetPriceFull, AssetPriceMapping, PriceProviderAsset};

use super::model::VerifiedToken;

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

pub fn map_token_asset(token: VerifiedToken) -> PriceProviderAsset {
    PriceProviderAsset::with_price(
        to_asset_price_mapping(&token.id),
        Some(map_token_market(&token)),
        Some(token.usd_price),
        Some(token.stats24h.price_change),
    )
}

pub fn map_token_price(mapping: AssetPriceMapping, token: &VerifiedToken) -> AssetPriceFull {
    AssetPriceFull::new(
        mapping,
        Price::new(token.usd_price, token.stats24h.price_change, chrono::Utc::now(), PriceProvider::Jupiter),
        Some(map_token_market(token)),
    )
}

fn map_token_market(token: &VerifiedToken) -> AssetMarket {
    AssetMarket {
        market_cap: token.mcap,
        market_cap_fdv: token.fdv,
        total_volume: token
            .stats24h
            .buy_volume
            .zip(token.stats24h.sell_volume)
            .map(|(buy_volume, sell_volume)| buy_volume + sell_volume),
        circulating_supply: token.circ_supply,
        total_supply: token.total_supply,
        ..AssetMarket::default()
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::TokenStats;
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

    #[test]
    fn test_map_token_price_maps_market_data() {
        let token = VerifiedToken {
            id: "token".to_string(),
            usd_price: 2.0,
            mcap: Some(10.0),
            fdv: Some(20.0),
            circ_supply: Some(5.0),
            total_supply: Some(10.0),
            stats24h: TokenStats {
                price_change: 3.0,
                buy_volume: Some(30.0),
                sell_volume: Some(40.0),
            },
        };

        let price = map_token_price(AssetPriceMapping::new(AssetId::from_token(Chain::Solana, "token"), "token".to_string()), &token);
        let market = price.market.unwrap();

        assert_eq!(price.price.price, 2.0);
        assert_eq!(price.price.price_change_percentage_24h, 3.0);
        assert_eq!(market.market_cap, Some(10.0));
        assert_eq!(market.market_cap_fdv, Some(20.0));
        assert_eq!(market.total_volume, Some(70.0));
        assert_eq!(market.circulating_supply, Some(5.0));
        assert_eq!(market.total_supply, Some(10.0));
    }
}
