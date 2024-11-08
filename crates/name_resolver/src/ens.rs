use crate::client::NameClient;
use crate::ens_provider::provider::Provider;
use async_trait::async_trait;
use gem_evm::address::EthereumAddress;
use primitives::{chain::Chain, name::NameProvider};
use std::{error::Error, str::FromStr};

pub struct ENSClient {
    provider: Provider,
}

impl ENSClient {
    pub fn new(url: String) -> Self {
        Self { provider: Provider::new(url) }
    }
}

#[async_trait]
impl NameClient for ENSClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Ens
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let address = self.provider.resolve_name(name, chain).await?;
        let address = EthereumAddress::from_str(&address)?.to_checksum();
        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["eth"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::SmartChain,
            Chain::Polygon,
            Chain::Optimism,
            Chain::Arbitrum,
            Chain::Base,
            Chain::AvalancheC,
            Chain::Fantom,
            Chain::Gnosis,
        ]
    }
}
