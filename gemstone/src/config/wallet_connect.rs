use primitives::Chain;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct WalletConnectConfig {
    pub chains: Vec<String>,
}

pub fn get_wallet_connect_config() -> WalletConnectConfig {
    let chains = vec![
        Chain::Ethereum,
        Chain::SmartChain,
        Chain::OpBNB,
        Chain::Base,
        Chain::AvalancheC,
        Chain::Polygon,
        Chain::Arbitrum,
        Chain::Optimism,
        Chain::Fantom,
        Chain::Gnosis,
        Chain::Solana,
        Chain::Manta,
        Chain::Blast,
        Chain::Mantle,
        Chain::ZkSync,
    ];

    WalletConnectConfig {
        chains: chains.into_iter().map(|x| x.to_string()).collect(),
    }
}
