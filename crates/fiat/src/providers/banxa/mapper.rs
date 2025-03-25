use primitives::Chain;

pub fn map_asset_chain(chain: String) -> Option<Chain> {
    match chain.as_str() {
        "BTC" => Some(Chain::Bitcoin),
        "LTC" => Some(Chain::Litecoin),
        "ETH" => Some(Chain::Ethereum),
        "TRON" => Some(Chain::Tron),
        "BSC" | "BNB" => Some(Chain::SmartChain),
        "SOL" => Some(Chain::Solana),
        "MATIC" => Some(Chain::Polygon),
        "ATOM" => Some(Chain::Cosmos),
        "AVAX-C" => Some(Chain::AvalancheC),
        "XRP" => Some(Chain::Xrp),
        "FTM" => Some(Chain::Fantom),
        "DOGE" => Some(Chain::Doge),
        "APT" => Some(Chain::Aptos),
        "TON" => Some(Chain::Ton),
        "SUI" => Some(Chain::Sui),
        "NEAR" => Some(Chain::Near),
        "CELO" => Some(Chain::Celo),
        "THORCHAIN" => Some(Chain::Thorchain),
        "XLM" => Some(Chain::Stellar),
        "ADA" => Some(Chain::Cardano),
        "DOT" => Some(Chain::Polkadot),
        "ALGO" => Some(Chain::Algorand),
        "ZKSYNC" => Some(Chain::ZkSync),
        "BCH" => Some(Chain::BitcoinCash),
        "WLD" => Some(Chain::World),
        "OPTIMISM" => Some(Chain::Optimism),
        "LINEA" => Some(Chain::Linea),
        "UNICHAIN" => Some(Chain::Unichain),
        "ARB" => Some(Chain::Arbitrum),
        "BASE" => Some(Chain::Base),
        "S" => Some(Chain::Sonic),
        "INJ" => Some(Chain::Injective),
        "MNT" => Some(Chain::Manta),
        _ => None,
    }
}
