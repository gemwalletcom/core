use std::{error::Error};
use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use sha2::{Sha256, Digest};

use primitives::name::{NameRecord, NameProvider};
use crate::client::NameClient;

#[derive(Debug, Deserialize, Serialize)]
pub struct EthscriptionResponse {
    pub result: bool,
    pub ethscription: EthscriptionDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EthscriptionDetails {
    pub current_owner: String,
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
    
    fn generate_content_uri(domain: &str) -> String {
        format!("data:,{}", domain)
    }
    
    fn generate_sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait]
impl NameClient for TreeClient {
    
    fn provider() -> NameProvider {
        NameProvider::Tree
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        if !name.ends_with(".tree") {
            return Err("Only .tree domains are supported".into());
        }

        if !matches!(chain, Chain::Ethereum | Chain::Polygon | Chain::SmartChain) {
            return Err(format!("Chain {} is not supported by .tree domains", chain).into());
        }
        
        let content_uri = Self::generate_content_uri(name);
        let sha256_val = Self::generate_sha256(&content_uri);
        let url = format!("{}/api/ethscriptions/exists/{}", self.api_url, sha256_val);
        
        let response: EthscriptionResponse = self.client.get(&url).send().await?.json().await?;
        let address = response.ethscription.current_owner;

        Ok(NameRecord { name: name.to_string(), chain, address, provider: Self::provider() })
    }

    fn domains() -> Vec<&'static str> {
        vec!["tree"]
    }

    fn chains() -> Vec<Chain> {
        vec![Chain::Ethereum, Chain::Polygon, Chain::SmartChain]
    }
}
