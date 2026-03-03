use alloy_ens::namehash;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use crate::error::NameError;
use primitives::{Chain, NameProvider};

const MAX_NAME_LENGTH: usize = 20;

pub struct InjectiveNameClient {
    url: String,
    client: Client,
}

// request
#[derive(Debug, Deserialize, Serialize)]
pub struct Resolver {
    resolver: ResolverNode,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverNode {
    node: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverAddress {
    address: ResolverNode,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverDataResponse {
    data: ResolverData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverData {
    address: String,
}

//const REGISTER_ADDRESS: &str = "inj1hm8vs8sr2h9nk0x66vctfs528wrp6k3gtgg275";
const RESOLVER_ADDRESS: &str = "inj1x9m0hceug9qylcyrrtwqtytslv2jrph433thgu";

impl InjectiveNameClient {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }
}

#[async_trait]
impl NameClient for InjectiveNameClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Injective
    }

    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let name_part = name.split('.').next().unwrap_or(name);
        if name_part.len() > MAX_NAME_LENGTH {
            return Err(Box::new(NameError::new(format!("name '{}' exceeds maximum length of {}", name_part, MAX_NAME_LENGTH))));
        }

        let hash = namehash(name);
        let resolve = ResolverAddress {
            address: ResolverNode { node: hash.to_vec() },
        };

        let string = serde_json::to_string(&resolve)?;
        let encoded = general_purpose::STANDARD.encode(string).to_string();

        let url = format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}", self.url, RESOLVER_ADDRESS, encoded);

        let response = self.client.get(&url).send().await?.json::<ResolverDataResponse>().await?;

        Ok(response.data.address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["inj"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Injective]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_rejects_long_name() {
        let client = InjectiveNameClient::new("https://localhost".to_string());
        let result = client.resolve("inj1kly3z4r8pzgfhh9cx5x69xjw0j4evlepq6ccgw.inj", Chain::Injective).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum length"));
    }

}
