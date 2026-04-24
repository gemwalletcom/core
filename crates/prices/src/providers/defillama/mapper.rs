use coingecko::get_coingecko_market_id_for_chain;
use primitives::{AssetId, Chain, PriceProvider};

use crate::{AssetPriceFull, AssetPriceMapping};

use super::model::CoinPrice;

pub fn to_defillama_id(asset_id: &AssetId) -> Option<String> {
    if asset_id.is_native() {
        let coingecko_id = get_coingecko_market_id_for_chain(asset_id.chain);
        if coingecko_id.is_empty() {
            return None;
        }
        return Some(format!("coingecko:{coingecko_id}"));
    }
    let slug = chain_to_defillama_slug(asset_id.chain)?;
    let token_id = asset_id.token_id.as_ref()?;
    Some(format!("{slug}:{token_id}"))
}

pub fn map_price(mapping: AssetPriceMapping, coin: &CoinPrice) -> AssetPriceFull {
    AssetPriceFull::simple(mapping, coin.price, 0.0, PriceProvider::DefiLlama)
}

fn chain_to_defillama_slug(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Ethereum => Some("ethereum"),
        Chain::Polygon => Some("polygon"),
        Chain::Arbitrum => Some("arbitrum"),
        Chain::Optimism => Some("optimism"),
        Chain::Base => Some("base"),
        Chain::SmartChain => Some("bsc"),
        Chain::AvalancheC => Some("avax"),
        Chain::Solana => Some("solana"),
        Chain::Tron => Some("tron"),
        Chain::Fantom => Some("fantom"),
        Chain::Gnosis => Some("xdai"),
        Chain::Blast => Some("blast"),
        Chain::Linea => Some("linea"),
        Chain::ZkSync => Some("era"),
        Chain::Mantle => Some("mantle"),
        Chain::Celo => Some("celo"),
        Chain::Sonic => Some("sonic"),
        Chain::Berachain => Some("berachain"),
        Chain::Unichain => Some("unichain"),
        Chain::OpBNB => Some("op_bnb"),
        Chain::Manta => Some("manta"),
        Chain::Ink => Some("ink"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_to_defillama_id() {
        let native_eth = AssetId::from_chain(Chain::Ethereum);
        assert_eq!(to_defillama_id(&native_eth).as_deref(), Some("coingecko:ethereum"));

        let usdc_eth = AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(to_defillama_id(&usdc_eth).as_deref(), Some("ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));

        let bsc_token = AssetId::from_token(Chain::SmartChain, "0x55d398326f99059fF775485246999027B3197955");
        assert_eq!(to_defillama_id(&bsc_token).as_deref(), Some("bsc:0x55d398326f99059fF775485246999027B3197955"));

        let unsupported = AssetId::from_token(Chain::Aptos, "0x1::aptos_coin::AptosCoin");
        assert_eq!(to_defillama_id(&unsupported), None);
    }

    #[test]
    fn test_map_price() {
        let response: super::super::model::PricesResponse = serde_json::from_str(include_str!("../../../testdata/defillama/prices.json")).unwrap();
        let coin = response.coins.get("coingecko:bitcoin").unwrap();
        let mapping = AssetPriceMapping::new(AssetId::from_chain(Chain::Bitcoin), "coingecko:bitcoin".to_string());

        let full = map_price(mapping, coin);

        assert_eq!(full.price.price, 67000.0);
        assert_eq!(full.price.price_change_percentage_24h, 0.0);
        assert_eq!(full.price.provider, PriceProvider::DefiLlama);
        assert!(full.market.is_none());
    }
}
