use async_trait::async_trait;
use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use primitives::name::{NameProvider, NameRecord};

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveRecord {
    pub owner: String,
}

pub struct EthsClient {
    api_url: String,
    client: Client,
}

impl EthsClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::new();
        Self { api_url, client }
    }
}

#[async_trait]
impl NameClient for EthsClient {
    fn provider() -> NameProvider {
        NameProvider::Tree
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let url = format!("{}/resolve/{}", self.api_url, name);
        let record: ResolveRecord = self.client.get(&url).send().await?.json().await?;
        let address = record.owner;

        Ok(NameRecord {
            name: name.to_string(),
            chain,
            address,
            provider: Self::provider(),
        })
    }

    fn domains() -> Vec<&'static str> {
        vec!["tree", "eths", "honk"]
    }

    fn chains() -> Vec<Chain> {
        vec![Chain::Ethereum, Chain::Polygon, Chain::SmartChain]
    }
}
