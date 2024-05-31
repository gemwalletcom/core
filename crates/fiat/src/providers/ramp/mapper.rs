use primitives::Chain;

pub fn map_asset_chain(chain: String) -> Option<Chain> {
    match chain.as_str() {
        "ETH" => Some(Chain::Ethereum),
        "SOLANA" => Some(Chain::Solana),
        "OPTIMISM" => Some(Chain::Optimism),
        "MATIC" => Some(Chain::Polygon),
        "XRP" => Some(Chain::Xrp),
        "TRON" => Some(Chain::Tron),
        "ARBITRUM" => Some(Chain::Arbitrum),
        "BASE" => Some(Chain::Base),
        "LTC" => Some(Chain::Litecoin),
        "AVAX" => Some(Chain::AvalancheC),
        "BSC" => Some(Chain::SmartChain),
        "COSMOS" => Some(Chain::Cosmos),
        "BTC" => Some(Chain::Bitcoin),
        "DOGE" => Some(Chain::Doge),
        "FANTOM" => Some(Chain::Fantom),
        "TON" => Some(Chain::Ton),
        "XDAI" => Some(Chain::Gnosis),
        "NEAR" => Some(Chain::Near),
        "ZKSYNCERA" => Some(Chain::ZkSync),
        "LINEA" => Some(Chain::Linea),
        "CELO" => Some(Chain::Celo),
        _ => None,
    }
}
