use std::collections::HashMap;
use std::sync::LazyLock;

use primitives::chain::Chain;

pub static COINGECKO_CHAIN_MAP: LazyLock<HashMap<String, Chain>> = LazyLock::new(|| {
    Chain::all()
        .iter()
        .map(|&x| (x, get_coingecko_market_id_for_chain(x).to_string()))
        .map(|(x, id)| (id, x))
        .collect()
});

// Full list https://api.coingecko.com/api/v3/asset_platforms
pub fn get_chain_for_coingecko_platform_id(id: &str) -> Option<Chain> {
    match id {
        "ethereum" => Some(Chain::Ethereum),
        "avalanche" => Some(Chain::AvalancheC),
        "abstract" => Some(Chain::Abstract),
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
        "sui" => Some(Chain::Sui),
        "opbnb" => Some(Chain::OpBNB),
        "mantle" => Some(Chain::Mantle),
        "celo" => Some(Chain::Celo),
        "zksync" => Some(Chain::ZkSync),
        "linea" => Some(Chain::Linea),
        "near" => Some(Chain::Near),
        "the-open-network" => Some(Chain::Ton),
        "algorand" => Some(Chain::Algorand),
        "berachain-bera" => Some(Chain::Berachain),
        "ink" => Some(Chain::Ink),
        "unichain" => Some(Chain::Unichain),
        "xrp" => Some(Chain::Xrp),
        "hyperliquid" => Some(Chain::Hyperliquid),
        "sonic" => Some(Chain::Sonic),
        "stellar" => Some(Chain::Stellar),
        "plasma" => Some(Chain::Plasma),
        _ => None,
    }
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
        Chain::Sonic => "sonic-3",
        Chain::Gnosis => "xdai",
        Chain::Celestia => "celestia",
        Chain::Injective => "injective-protocol",
        Chain::Sei => "sei-network",
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
    }
}
