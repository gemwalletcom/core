use crate::client::NameClient;
use async_trait::async_trait;
use primitives::{Chain, NameProvider};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveDomain {
    pub s: String,
    pub result: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveRecord {
    pub s: String,
    pub result: ResolveRecordResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveRecordResult {
    pub deserialized: String,
}

pub struct SNSClient {
    url: String,
    client: Client,
}

impl SNSClient {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }

    async fn resolve_hex_address(&self, name: &str, record: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/record-v2/{}/{}", self.url, name, record);
        let response = self.client.get(&url).send().await?.json::<ResolveRecord>().await?;

        if response.s != "ok" {
            return Err("error".to_string().into());
        }

        Ok(response.result.deserialized)
    }

    async fn resolve_sol_address(&self, name: &str, _chain: &Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/resolve/{}", self.url, name);
        let response = self.client.get(&url).send().await?.json::<ResolveDomain>().await?;

        if response.s != "ok" {
            return Err("error".to_string().into());
        }
        Ok(response.result)
    }
}

#[async_trait]
impl NameClient for SNSClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Sns
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::Solana => {
                return self.resolve_sol_address(name, &chain.clone()).await;
            }
            Chain::SmartChain => {
                return self.resolve_hex_address(name, "BSC").await;
            }
            _ => return Err("error".to_string().into()),
        }
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sol"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Solana, Chain::SmartChain]
    }
}
