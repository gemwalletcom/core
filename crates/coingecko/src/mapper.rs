use primitives::chain::Chain;

const COINGECKO_CHAIN_PLATFORMS: &[(Chain, &str)] = &[
    (Chain::Ethereum, "ethereum"),
    (Chain::AvalancheC, "avalanche"),
    (Chain::Abstract, "abstract"),
    (Chain::Optimism, "optimistic-ethereum"),
    (Chain::Base, "base"),
    (Chain::Arbitrum, "arbitrum-one"),
    (Chain::SmartChain, "binance-smart-chain"),
    (Chain::Manta, "manta-pacific"),
    (Chain::Tron, "tron"),
    (Chain::Polygon, "polygon-pos"),
    (Chain::Solana, "solana"),
    (Chain::Blast, "blast"),
    (Chain::Gnosis, "xdai"),
    (Chain::Fantom, "fantom"),
    (Chain::Osmosis, "osmosis"),
    (Chain::Cosmos, "cosmos"),
    (Chain::Aptos, "aptos"),
    (Chain::Sui, "sui"),
    (Chain::OpBNB, "opbnb"),
    (Chain::Mantle, "mantle"),
    (Chain::Celo, "celo"),
    (Chain::ZkSync, "zksync"),
    (Chain::Linea, "linea"),
    (Chain::Near, "near"),
    (Chain::Ton, "the-open-network"),
    (Chain::Algorand, "algorand"),
    (Chain::Berachain, "berachain-bera"),
    (Chain::Ink, "ink"),
    (Chain::Unichain, "unichain"),
    (Chain::SeiEvm, "sei-network"),
    (Chain::Xrp, "xrp"),
    (Chain::Hyperliquid, "hyperevm"),
    (Chain::Sonic, "sonic"),
    (Chain::Stellar, "stellar"),
    (Chain::Plasma, "plasma"),
    (Chain::Monad, "monad"),
    (Chain::Stable, "stable-2"),
];

pub fn get_chains_for_coingecko_market_id(id: &str) -> Vec<Chain> {
    Chain::all().into_iter().filter(|chain| get_coingecko_market_id_for_chain(*chain) == id).collect()
}

// Full list https://api.coingecko.com/api/v3/asset_platforms
pub fn get_chain_for_coingecko_platform_id(id: &str) -> Option<Chain> {
    COINGECKO_CHAIN_PLATFORMS.iter().find_map(|(chain, platform_id)| (*platform_id == id).then_some(*chain))
}

pub fn get_coingecko_platform_id_for_chain(chain: Chain) -> Option<&'static str> {
    COINGECKO_CHAIN_PLATFORMS
        .iter()
        .find_map(|(candidate, platform_id)| (*candidate == chain).then_some(*platform_id))
}

pub fn get_coingecko_market_id_for_chain(chain: Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "bitcoin",
        Chain::BitcoinCash => "bitcoin-cash",
        Chain::Litecoin => "litecoin",
        Chain::Ethereum
        | Chain::Base
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::ZkSync
        | Chain::Blast
        | Chain::Linea
        | Chain::Manta
        | Chain::World
        | Chain::Abstract
        | Chain::Ink
        | Chain::Unichain => "ethereum",
        Chain::SmartChain | Chain::OpBNB => "binancecoin",
        Chain::Solana => "solana",
        Chain::Polygon => "polygon-ecosystem-token",
        Chain::Thorchain => "thorchain",
        Chain::Cosmos => "cosmos",
        Chain::Osmosis => "osmosis",
        Chain::Ton => "the-open-network",
        Chain::Tron => "tron",
        Chain::Doge => "dogecoin",
        Chain::Zcash => "zcash",
        Chain::Aptos => "aptos",
        Chain::AvalancheC => "avalanche-2",
        Chain::Sui => "sui",
        Chain::Xrp => "ripple",
        Chain::Fantom => "fantom",
        Chain::Sonic => "sonic-3",
        Chain::Gnosis => "xdai",
        Chain::Celestia => "celestia",
        Chain::Injective => "injective-protocol",
        Chain::Sei => "sei-network",
        Chain::SeiEvm => "sei-network",
        Chain::Noble => "usd-coin",
        Chain::Mantle => "mantle",
        Chain::Celo => "celo",
        Chain::Near => "near",
        Chain::Stellar => "stellar",
        Chain::Algorand => "algorand",
        Chain::Polkadot => "polkadot",
        Chain::Cardano => "cardano",
        Chain::Berachain => "berachain-bera",
        Chain::Hyperliquid => "hyperliquid",
        Chain::HyperCore => "hyperliquid",
        Chain::Monad => "monad",
        Chain::Plasma => "plasma",
        Chain::XLayer => "okb",
        Chain::Stable => "tether", // gUSDT is the native gas token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coingecko_platform_mapping() {
        assert_eq!(get_chain_for_coingecko_platform_id("binance-smart-chain"), Some(Chain::SmartChain));
        assert_eq!(get_coingecko_platform_id_for_chain(Chain::SmartChain), Some("binance-smart-chain"));
        assert_eq!(get_chain_for_coingecko_platform_id("unknown"), None);
        assert_eq!(get_coingecko_platform_id_for_chain(Chain::Bitcoin), None);

        for (chain, platform_id) in COINGECKO_CHAIN_PLATFORMS {
            assert_eq!(get_chain_for_coingecko_platform_id(platform_id), Some(*chain));
            assert_eq!(get_coingecko_platform_id_for_chain(*chain), Some(*platform_id));
        }
    }
}
