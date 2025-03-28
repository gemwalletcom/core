use primitives::Chain;

use super::model::Asset;

pub fn map_asset_chain(asset: Asset) -> Option<Chain> {
    match asset.metadata?.network_code.as_str() {
        "ethereum" => Some(Chain::Ethereum),
        "binance_smart_chain" | "bnb_chain" => Some(Chain::SmartChain),
        "solana" => Some(Chain::Solana),
        "arbitrum" => Some(Chain::Arbitrum),
        "base" => Some(Chain::Base),
        "avalanche_c_chain" => Some(Chain::AvalancheC),
        "optimism" => Some(Chain::Optimism),
        "polygon" => Some(Chain::Polygon),
        "tron" => Some(Chain::Tron),
        "aptos" => Some(Chain::Aptos),
        "bitcoin" => Some(Chain::Bitcoin),
        "dogecoin" => Some(Chain::Doge),
        "litecoin" => Some(Chain::Litecoin),
        "ripple" => Some(Chain::Xrp),
        "sui" => Some(Chain::Sui),
        "ton" => Some(Chain::Ton),
        "cosmos" => Some(Chain::Cosmos),
        "near" => Some(Chain::Near),
        "linea" => Some(Chain::Linea),
        "zksync" => Some(Chain::ZkSync),
        "celo" => Some(Chain::Celo),
        "stellar" => Some(Chain::Stellar),
        "algorand" => Some(Chain::Algorand),
        "polkadot" => Some(Chain::Polkadot),
        "osmosis" => Some(Chain::Osmosis),
        "fantom" => Some(Chain::Fantom),
        "celestia" => Some(Chain::Celestia),
        "sei" => Some(Chain::Sei),
        "injective" => Some(Chain::Injective),
        "noble" => Some(Chain::Noble),
        "worldchain" => Some(Chain::World),
        "cardano" => Some(Chain::Cardano),
        "bitcoin_cash" => Some(Chain::BitcoinCash),
        "mantle" => Some(Chain::Mantle),
        "sonic" => Some(Chain::Sonic),
        _ => None,
    }
}
