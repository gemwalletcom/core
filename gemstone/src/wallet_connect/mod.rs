use primitives::Chain;
use primitives::WallletConnectCAIP2;

// CAIP-2 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md
pub fn get_namespace(chain: Chain) -> Option<String> {
    match chain {
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Base
        | Chain::AvalancheC
        | Chain::OpBNB
        | Chain::Gnosis
        | Chain::Optimism
        | Chain::Fantom
        | Chain::Manta
        | Chain::Blast => Some(WallletConnectCAIP2::Eip155.as_ref().to_string()),
        Chain::Solana => Some(WallletConnectCAIP2::Solana.as_ref().to_string()),
        Chain::Bitcoin
        | Chain::Litecoin
        | Chain::Binance
        | Chain::Thorchain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei => None,
    }
}

// CAIP-20 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-20.md
pub fn get_reference(chain: Chain) -> Option<String> {
    match chain {
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Base
        | Chain::AvalancheC
        | Chain::OpBNB
        | Chain::Gnosis
        | Chain::Optimism
        | Chain::Fantom
        | Chain::Manta
        | Chain::Blast => Some(chain.network_id().to_string()),
        Chain::Solana => Some("4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ".to_string()),
        Chain::Bitcoin
        | Chain::Litecoin
        | Chain::Binance
        | Chain::Thorchain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei => None,
    }
}
