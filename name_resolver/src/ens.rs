use crate::client::NameClient;
use crate::ens_provider::provider::Provider;
use async_trait::async_trait;
use primitives::{
    chain::Chain,
    name::{NameProvider, NameRecord},
    EthereumAddress,
};
use std::{error::Error, str::FromStr};

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
            address: EthereumAddress::from_str(&address)?.to_checksum(),
            provider: Self::provider().as_ref().to_string(),
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
