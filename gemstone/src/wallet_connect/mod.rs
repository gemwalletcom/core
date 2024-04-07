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
        | Chain::Blast
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo => Some(WallletConnectCAIP2::Eip155.as_ref().to_string()),
        Chain::Solana => Some(WallletConnectCAIP2::Solana.as_ref().to_string()),
        Chain::Cosmos
        | Chain::Osmosis
        | Chain::Celestia
        | Chain::Injective
        | Chain::Noble
        | Chain::Sei => Some(format!(
            "{}:{}",
            WallletConnectCAIP2::Cosmos.as_ref(),
            chain.network_id()
        )), // cosmos:cosmoshub-4
        Chain::Bitcoin
        | Chain::Litecoin
        | Chain::Binance
        | Chain::Thorchain
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp => None,
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
        | Chain::Blast
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo => Some(chain.network_id().to_string()),
        Chain::Solana => Some("4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ".to_string()),
        Chain::Cosmos
        | Chain::Osmosis
        | Chain::Celestia
        | Chain::Noble
        | Chain::Sei
        | Chain::Injective => {
            get_namespace(chain).map(|namespace| format!("{}:{}", namespace, chain.network_id()))
        }
        Chain::Bitcoin
        | Chain::Litecoin
        | Chain::Binance
        | Chain::Thorchain
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp => None,
    }
}
