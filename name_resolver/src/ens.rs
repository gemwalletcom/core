use crate::client::NameClient;
use crate::ens_provider::provider::Provider;
use async_trait::async_trait;
use primitives::{
    chain::Chain,
    name::{NameProvider, NameRecord},
};
use std::error::Error;

pub struct ENSClient {
    provider: Provider,
}

impl ENSClient {
    pub fn new(url: String) -> Self {
        Self {
            provider: Provider::new(url),
        }
    }
}

#[async_trait]
impl NameClient for ENSClient {
    fn provider() -> NameProvider {
        NameProvider::Ens
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let address = self.provider.resolve_name(name, chain).await?;
        Ok(NameRecord {
            name: name.to_string(),
            chain,
            address,
            provider: Self::provider(),
        })
    }

    fn domains() -> Vec<&'static str> {
        vec!["eth"]
    }

    fn chains() -> Vec<Chain> {
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
