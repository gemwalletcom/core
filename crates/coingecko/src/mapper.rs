use primitives::chain::Chain;

pub fn get_chain_for_coingecko_platform_id(id: &str) -> Option<Chain> {
    match id {
        "ethereum" => Some(Chain::Ethereum),
        "avalanche" => Some(Chain::AvalancheC),
        "optimistic-ethereum" => Some(Chain::Optimism),
        "base" => Some(Chain::Base),
        "arbitrum-one" => Some(Chain::Arbitrum),
        "binance-smart-chain" => Some(Chain::SmartChain),
        "manta-pacific" => Some(Chain::Manta),
        "tron" => Some(Chain::Tron),
        "polygon-pos" => Some(Chain::Polygon),
        "solana" => Some(Chain::Solana),
        "blast" => Some(Chain::Blast),
        "xdai" => Some(Chain::Gnosis),
        "fantom" => Some(Chain::Fantom),
        "osmosis" => Some(Chain::Osmosis),
        "cosmos" => Some(Chain::Cosmos),
        "aptos" => Some(Chain::Aptos),
        "suiche" => Some(Chain::Sui),
        "opbnb" => Some(Chain::OpBNB),
        _ => None,
    }
}

pub fn get_chain_for_coingecko_id(id: &str) -> Option<Chain> {
    match id {
        "bitcoin" => Some(Chain::Bitcoin),
        "litecoin" => Some(Chain::Litecoin),
        "binancecoin" => Some(Chain::Binance),
        "ethereum" => Some(Chain::Ethereum),
        "matic-network" => Some(Chain::Polygon),
        "solana" => Some(Chain::Solana),
        "thorchain" => Some(Chain::Thorchain),
        "cosmos" => Some(Chain::Cosmos),
        "osmosis" => Some(Chain::Osmosis),
        "the-open-network" => Some(Chain::Ton),
        "tron" => Some(Chain::Tron),
        "dogecoin" => Some(Chain::Doge),
        "aptos" => Some(Chain::Aptos),
        "avalanche-2" => Some(Chain::AvalancheC),
        "sui" => Some(Chain::Sui),
        "ripple" => Some(Chain::Xrp),
        "gnosis" => Some(Chain::Gnosis),
        "fantom" => Some(Chain::Fantom),
        "celestia" => Some(Chain::Celestia),
        "injective" | "injective-protocol" => Some(Chain::Injective),
        "sei-network" => Some(Chain::Sei),
        "manta-network" => Some(Chain::Manta),
        _ => None,
    }
}

// mapping between l2  to l1 chains
pub fn get_associated_chains(chain: Chain) -> Vec<Chain> {
    match chain {
        Chain::Binance => {
            vec![Chain::SmartChain, Chain::OpBNB]
        }
        Chain::Ethereum => {
            vec![
                Chain::Arbitrum,
                Chain::Optimism,
                Chain::Base,
                Chain::Manta,
                Chain::Blast,
            ]
        }
        _ => {
            vec![]
        }
    }
}
