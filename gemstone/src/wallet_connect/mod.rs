use primitives::Chain;
use primitives::WalletConnectCAIP2;
use std::str::FromStr;

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
        | Chain::Celo
        | Chain::World
        | Chain::Sonic
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink => Some(WalletConnectCAIP2::Eip155.as_ref().to_string()),
        Chain::Solana => Some(WalletConnectCAIP2::Solana.as_ref().to_string()),
        Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Injective | Chain::Noble | Chain::Sei => {
            Some(format!("{}:{}", WalletConnectCAIP2::Cosmos.as_ref(), chain.network_id()))
        } // cosmos:cosmoshub-4
        Chain::Algorand => Some(WalletConnectCAIP2::Algorand.as_ref().to_string()),
        Chain::Bitcoin
        | Chain::Litecoin
        | Chain::Thorchain
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp
        | Chain::Near
        | Chain::Stellar
        | Chain::BitcoinCash
        | Chain::Polkadot
        | Chain::Cardano => None,
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
        | Chain::Celo
        | Chain::Bitcoin
        | Chain::Litecoin
        | Chain::Doge
        | Chain::World
        | Chain::Sonic
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink => Some(chain.network_id().to_string()),
        Chain::Solana => Some("4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ".to_string()),
        Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Noble | Chain::Sei | Chain::Injective | Chain::Thorchain => {
            get_namespace(chain).map(|namespace| format!("{}:{}", namespace, chain.network_id()))
        }
        Chain::Algorand => Some("wGHE2Pwdvd7S12BL5FaOP20EGYesN73k".to_string()),
        Chain::Ton
        | Chain::Tron
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp
        | Chain::Near
        | Chain::Stellar
        | Chain::BitcoinCash
        | Chain::Polkadot
        | Chain::Cardano => None,
    }
}

#[derive(uniffi::Object)]
struct WalletConnectNamespace {}

#[uniffi::export]
impl WalletConnectNamespace {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn get_namespace(&self, chain: String) -> Option<String> {
        let chain = Chain::from_str(&chain).ok()?;
        get_namespace(chain)
    }

    fn get_reference(&self, chain: String) -> Option<String> {
        let chain = Chain::from_str(&chain).ok()?;
        get_reference(chain)
    }
}
