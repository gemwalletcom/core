use primitives::chain::Chain;

// Full list https://api.coingecko.com/api/v3/asset_platforms
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
        "mantle" => Some(Chain::Mantle),
        "celo" => Some(Chain::Celo),
        "zksync" => Some(Chain::ZkSync),
        "linea" => Some(Chain::Linea),
        "near" => Some(Chain::Near),
        _ => None,
    }
}

pub fn get_coingecko_market_id_for_chain(chain: Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "bitcoin",
        Chain::Litecoin => "litecoin",
        Chain::Ethereum
        | Chain::Base
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::ZkSync
        | Chain::Blast => "ethereum",
        Chain::Binance | Chain::OpBNB | Chain::SmartChain => "binancecoin",
        Chain::Solana => "solana",
        Chain::Polygon => "matic-network",
        Chain::Thorchain => "thorchain",
        Chain::Cosmos => "cosmos",
        Chain::Osmosis => "osmosis",
        Chain::Ton => "the-open-network",
        Chain::Tron => "tron",
        Chain::Doge => "dogecoin",
        Chain::Aptos => "aptos",
        Chain::AvalancheC => "avalanche-2",
        Chain::Sui => "sui",
        Chain::Xrp => "ripple",
        Chain::Fantom => "fantom",
        Chain::Gnosis => "gnosis",
        Chain::Celestia => "celestia",
        Chain::Injective => "injective",
        Chain::Sei => "sei-network",
        Chain::Manta => "manta-network",
        Chain::Noble => "usd-coin",
        Chain::Linea => "linea",
        Chain::Mantle => "mantle",
        Chain::Celo => "celo",
        Chain::Near => "near",
    }
}
