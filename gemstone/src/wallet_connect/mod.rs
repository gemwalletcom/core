use primitives::WalletConnectCAIP2;
use primitives::{Chain, ChainType};
use std::str::FromStr;

// CAIP-2 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md
pub fn get_namespace(chain: Chain) -> Option<String> {
    match chain.chain_type() {
        ChainType::Ethereum => Some(WalletConnectCAIP2::Eip155.as_ref().to_string()),
        ChainType::Solana => Some(WalletConnectCAIP2::Solana.as_ref().to_string()),
        ChainType::Cosmos => Some(format!("{}:{}", WalletConnectCAIP2::Cosmos.as_ref(), chain.network_id())),
        ChainType::Algorand => Some(WalletConnectCAIP2::Algorand.as_ref().to_string()),
        ChainType::Bitcoin
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Polkadot
        | ChainType::Cardano => None,
    }
}

pub fn get_chain_type(caip2: String) -> Option<ChainType> {
    let caip2 = WalletConnectCAIP2::from_str(&caip2).ok()?;
    match caip2 {
        WalletConnectCAIP2::Eip155 => Some(ChainType::Ethereum),
        WalletConnectCAIP2::Solana => Some(ChainType::Solana),
        WalletConnectCAIP2::Cosmos => Some(ChainType::Cosmos),
        WalletConnectCAIP2::Algorand => Some(ChainType::Algorand),
    }
}

// CAIP-20 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-20.md
pub fn get_reference(chain: Chain) -> Option<String> {
    match chain.chain_type() {
        ChainType::Ethereum => Some(chain.network_id().to_string()),
        ChainType::Solana => Some("4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ".to_string()),
        ChainType::Cosmos => get_namespace(chain).map(|namespace| format!("{}:{}", namespace, chain.network_id())),
        ChainType::Algorand => Some("wGHE2Pwdvd7S12BL5FaOP20EGYesN73k".to_string()),
        ChainType::Bitcoin
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Polkadot
        | ChainType::Cardano => None,
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

    fn get_chain_type(&self, caip2: String) -> Option<String> {
        Some(get_chain_type(caip2)?.as_ref().to_string())
    }
}
