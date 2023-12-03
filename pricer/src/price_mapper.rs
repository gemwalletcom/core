use primitives::chain::Chain;

pub fn get_chain_for_coingecko_id(id: &str) -> Option<Chain> {
    match id {
        "bitcoin" => Some(Chain::Bitcoin),
        "litecoin" => Some(Chain::Litecoin),
        "binancecoin" => Some(Chain::Binance),
        "ethereum" => Some(Chain::Ethereum),
        "binance-smart-chain" => Some(Chain::SmartChain),
        "matic-network" |
        "polygon-pos" => Some(Chain::Polygon),
        "solana" => Some(Chain::Solana),
        "arbitrum-one" => Some(Chain::Arbitrum),
        "optimistic-ethereum" => Some(Chain::Optimism),
        "thorchain" => Some(Chain::Thorchain),
        "cosmos" => Some(Chain::Cosmos),
        "osmosis" => Some(Chain::Osmosis),
        "the-open-network" => Some(Chain::Ton),
        "tron" => Some(Chain::Tron),
        "dogecoin" => Some(Chain::Doge),
        "aptos" => Some(Chain::Aptos),
        "avalanche-2" => Some(Chain::AvalancheC),
        "sui" => Some(Chain::Sui),
        "ripple" => Some(Chain::Ripple),
        "gnosis" => Some(Chain::Gnosis),
        "fantom" => Some(Chain::Fantom),
        "celestia" => Some(Chain::Celestia),
        _ => {
            None
        }
    }
}