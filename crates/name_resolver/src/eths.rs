use async_trait::async_trait;
use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use primitives::NameProvider;

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
    fn provider(&self) -> NameProvider {
        NameProvider::Tree
    }

    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/resolve/{}", self.api_url, name);
        let record: ResolveRecord = self.client.get(&url).send().await?.json().await?;
        let address = record.owner;

        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["tree", "eths", "honk"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Ethereum, Chain::Polygon, Chain::SmartChain]
    }
}
