use primitives::{Chain, EVMChain};

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct WalletConnectConfig {
    pub chains: Vec<String>,
}

pub fn get_wallet_connect_config() -> WalletConnectConfig {
    let chains: Vec<Chain> = [vec![Chain::Solana, Chain::Sui], EVMChain::all().iter().map(|x| x.to_chain()).collect()].concat();

    WalletConnectConfig {
        chains: chains.into_iter().map(|x| x.to_string()).collect(),
    }
}
