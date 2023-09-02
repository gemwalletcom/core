use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::{error::Error};

use primitives::name::{NameRecord, NameProvider};
use crate::client::NameClient;

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolveRecord {
    pub owner: String,
}

pub struct TreeClient {
    api_url: String,
    client: Client,
}

impl TreeClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::new();
        Self {
            api_url,
            client,
        }
    }
}

#[async_trait]
impl NameClient for TreeClient {
    
    fn provider() -> NameProvider {
        NameProvider::Tree
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let url = format!("{}/resolve/{}", self.api_url, name);
        let record: ResolveRecord = self.client.get(&url).send().await?.json().await?;
        let address = record.owner;

        Ok(NameRecord { name: name.to_string(), chain, address, provider: Self::provider() })
    }

    fn domains() -> Vec<&'static str> {
        vec!["tree"]
    }

    fn chains() -> Vec<Chain> {
        vec![
            Chain::Ethereum, 
            Chain::Polygon, 
            Chain::SmartChain
        ]
    }
}
