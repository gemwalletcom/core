use crate::client::NameClient;
use async_trait::async_trait;
use primitives::{
    chain::Chain,
    name::{NameProvider, NameRecord},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveName {
    pub address: String,
}

pub struct AptosClient {
    url: String,
    client: Client,
}

impl AptosClient {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }

    async fn resolve_name(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/api/mainnet/v1/address/{}", self.url, name);
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<ResolveName>()
            .await?;

        Ok(response.address)
    }
}

#[async_trait]
impl NameClient for AptosClient {
    fn provider() -> NameProvider {
        NameProvider::Aptos
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        match chain {
            Chain::Aptos => {
                let address = self.resolve_name(name).await?;
                Ok(NameRecord {
                    name: name.to_string(),
                    chain,
                    address,
                    provider: NameProvider::Ud, //Self::provider(),
                })
            }
            _ => return Err("error".to_string().into()),
        }
    }

    fn domains() -> Vec<&'static str> {
        vec!["apt"]
    }

    fn chains() -> Vec<Chain> {
        vec![Chain::Aptos]
    }
}
