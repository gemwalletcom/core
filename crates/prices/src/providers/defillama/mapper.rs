use coingecko::get_coingecko_market_id_for_chain;
use primitives::{AssetId, Chain, PriceProvider};

use crate::{AssetPriceFull, AssetPriceMapping};

use super::model::CoinPrice;

const DEFILLAMA_CHAIN_SLUGS: &[(Chain, &str)] = &[
    (Chain::Ethereum, "ethereum"),
    (Chain::Polygon, "polygon"),
    (Chain::Arbitrum, "arbitrum"),
    (Chain::Optimism, "optimism"),
    (Chain::Base, "base"),
    (Chain::SmartChain, "bsc"),
    (Chain::AvalancheC, "avax"),
    (Chain::Solana, "solana"),
    (Chain::Tron, "tron"),
    (Chain::Fantom, "fantom"),
    (Chain::Gnosis, "xdai"),
    (Chain::Blast, "blast"),
    (Chain::Linea, "linea"),
    (Chain::ZkSync, "era"),
    (Chain::Mantle, "mantle"),
    (Chain::Celo, "celo"),
    (Chain::Sonic, "sonic"),
    (Chain::Berachain, "berachain"),
    (Chain::Unichain, "unichain"),
    (Chain::OpBNB, "op_bnb"),
    (Chain::Manta, "manta"),
    (Chain::Ink, "ink"),
];

pub fn defillama_id_for_asset_id(asset_id: &AssetId) -> Option<String> {
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

pub fn asset_ids_for_defillama_id(provider_price_id: &str) -> Vec<AssetId> {
    if let Some(coingecko_id) = provider_price_id.strip_prefix("coingecko:") {
        return coingecko::get_chains_for_coingecko_market_id(coingecko_id).into_iter().map(AssetId::from_chain).collect();
    }

    let Some((slug, token_id)) = provider_price_id.split_once(':') else {
        return vec![];
    };
    chain_from_defillama_slug(slug)
        .map(|chain| vec![AssetId::from(chain, Some(token_id.to_string()))])
        .unwrap_or_default()
}

pub fn map_price(mapping: AssetPriceMapping, coin: &CoinPrice) -> AssetPriceFull {
    AssetPriceFull::simple(mapping, coin.price, 0.0, PriceProvider::DefiLlama)
}

fn chain_to_defillama_slug(chain: Chain) -> Option<&'static str> {
    DEFILLAMA_CHAIN_SLUGS.iter().find_map(|(candidate, slug)| (*candidate == chain).then_some(*slug))
}

fn chain_from_defillama_slug(slug: &str) -> Option<Chain> {
    DEFILLAMA_CHAIN_SLUGS.iter().find_map(|(chain, candidate)| (*candidate == slug).then_some(*chain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_defillama_id_for_asset_id() {
        let native_eth = AssetId::from_chain(Chain::Ethereum);
        assert_eq!(defillama_id_for_asset_id(&native_eth).as_deref(), Some("coingecko:ethereum"));

        let usdc_eth = AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(defillama_id_for_asset_id(&usdc_eth).as_deref(), Some("ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));

        let bsc_token = AssetId::from_token(Chain::SmartChain, "0x55d398326f99059fF775485246999027B3197955");
        assert_eq!(defillama_id_for_asset_id(&bsc_token).as_deref(), Some("bsc:0x55d398326f99059fF775485246999027B3197955"));

        let unsupported = AssetId::from_token(Chain::Aptos, "0x1::aptos_coin::AptosCoin");
        assert_eq!(defillama_id_for_asset_id(&unsupported), None);

        assert_eq!(asset_ids_for_defillama_id("bsc:0x55d398326f99059fF775485246999027B3197955"), vec![bsc_token]);
        assert_eq!(asset_ids_for_defillama_id("unknown:0x1"), Vec::<AssetId>::new());

        for (chain, slug) in DEFILLAMA_CHAIN_SLUGS {
            assert_eq!(chain_to_defillama_slug(*chain), Some(*slug));
            assert_eq!(chain_from_defillama_slug(slug), Some(*chain));
        }
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
