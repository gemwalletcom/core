use std::error::Error;
use ethers::providers::*;
use async_trait::async_trait;
use primitives::{chain::Chain, name::{NameRecord, NameProvider}};
use crate::client::NameClient;

pub struct ENSClient {
    url: String
}

impl ENSClient {
    pub fn new(
        url: String
    ) -> Self {
        Self {
            url
        }
    }
}

#[async_trait]
impl NameClient for ENSClient {
   
    fn provider() -> NameProvider {
        NameProvider::Ens
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let provider = Provider::<Http>::try_from(&self.url).unwrap();
        let address = provider.resolve_name(name).await?;
        return Ok(NameRecord { name: name.to_string(), chain, address: format!("{:?}",address), provider: Self::provider() })
    }

    fn domains() -> Vec<&'static str> {
        vec![
            "eth"
        ]
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
        ]
    }
}