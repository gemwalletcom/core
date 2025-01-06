use primitives::Chain;

use super::model::Asset;

pub fn map_asset_chain(asset: Asset) -> Option<Chain> {
    match asset.network.name.as_str() {
        "ethereum" => Some(Chain::Ethereum),
        "polygon" => Some(Chain::Polygon),
        "aptos" => Some(Chain::Aptos),
        "sui" => Some(Chain::Sui),
        "arbitrum" => Some(Chain::Arbitrum),
        "optimism" => Some(Chain::Optimism),
        "base" => Some(Chain::Base),
        "bsc" => Some(Chain::SmartChain),
        "tron" => Some(Chain::Tron),
        "solana" => Some(Chain::Solana),
        "avaxcchain" => Some(Chain::AvalancheC),
        "ton" => Some(Chain::Ton),
        "osmosis" => Some(Chain::Osmosis),
        "fantom" => Some(Chain::Fantom),
        "injective" => Some(Chain::Injective),
        "sei" => Some(Chain::Sei),
        "linea" => Some(Chain::Linea),
        "zksync" => Some(Chain::ZkSync),
        "celo" => Some(Chain::Celo),
        "mantle" => Some(Chain::Mantle),
        "opbnb" => Some(Chain::OpBNB),
        "mainnet" => match asset.coin_id.as_str() {
            "bitcoin" => Some(Chain::Bitcoin),
            "litecoin" => Some(Chain::Litecoin),
            "ripple" => Some(Chain::Xrp),
            "dogecoin" => Some(Chain::Doge),
            "tron" => Some(Chain::Tron),
            "cosmos" => Some(Chain::Cosmos),
            "near" => Some(Chain::Near),
            "stellar" => Some(Chain::Stellar),
            "algorand" => Some(Chain::Algorand),
            "polkadot" => Some(Chain::Polkadot),
            "cardano" => Some(Chain::Cardano),
            _ => None,
        },
        _ => None,
    }
}
