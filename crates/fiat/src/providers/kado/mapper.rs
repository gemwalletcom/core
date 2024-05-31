use primitives::Chain;

pub fn map_asset_chain(chain: String) -> Option<Chain> {
    match chain.as_str() {
        "bitcoin" => Some(Chain::Bitcoin),
        "litecoin" => Some(Chain::Litecoin),
        "ethereum" => Some(Chain::Ethereum),
        "optimism" | "Optimism" => Some(Chain::Optimism),
        "polygon" | "Polygon" => Some(Chain::Polygon),
        "base" => Some(Chain::Base),
        "arbitrum" | "Arbitrum" => Some(Chain::Arbitrum),
        "avalanche" | "Avalanche" => Some(Chain::AvalancheC),
        "solana" => Some(Chain::Solana),
        "osmosis" => Some(Chain::Osmosis),
        "cosmos hub" => Some(Chain::Cosmos),
        "ripple" => Some(Chain::Xrp),
        "celestia" => Some(Chain::Celestia),
        "injective" => Some(Chain::Injective),
        "sei" => Some(Chain::Sei),
        "noble" => Some(Chain::Noble),
        _ => None,
    }
}
