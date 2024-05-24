use primitives::Chain;

pub fn chain_transaction_timeout_seconds(chain: Chain) -> f64 {
    match chain {
        Chain::Bitcoin => 28800_f64,
        Chain::Litecoin | Chain::Doge => 7200_f64,
        Chain::Solana => 300_f64,
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Thorchain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Arbitrum
        | Chain::Ton
        | Chain::Tron
        | Chain::Optimism
        | Chain::Aptos
        | Chain::Base
        | Chain::AvalancheC
        | Chain::Sui
        | Chain::Xrp
        | Chain::OpBNB
        | Chain::Fantom
        | Chain::Gnosis
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei
        | Chain::Manta
        | Chain::Blast
        | Chain::Noble
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo
        | Chain::Near => 1800_f64, // 30 minutes
    }
}
