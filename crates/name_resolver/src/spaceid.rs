use async_trait::async_trait;
use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use primitives::NameProvider;

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveRecord {
    pub code: i32,
    pub address: String,
}

pub struct SpaceIdClient {
    api_url: String,
    client: Client,
}

impl SpaceIdClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::new();
        Self { api_url, client }
    }
}

#[async_trait]
impl NameClient for SpaceIdClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Spaceid
    }

    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let tld = name.split('.').clone().last().unwrap_or_default();
        let url = format!("{}/v1/getAddress?tld={}&domain={}", self.api_url, tld, name);
        let record: ResolveRecord = self.client.get(&url).send().await?.json().await?;
        if record.code != 0 {
            return Err("SpaceIdClient: code != 0".into());
        }
        let address = record.address;

        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["bnb", "arb"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::SmartChain, Chain::Arbitrum]
    }
}
