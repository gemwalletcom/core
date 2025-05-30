use async_trait::async_trait;
use primitives::{name::NameProvider, Chain};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde::Deserialize;
use std::error::Error;

use crate::client::NameClient;

pub const DEFAULT_API_URL_BASE: &str = "https://api.hlnames.xyz";
const API_KEY_HEADER: &str = "X-API-Key";

#[derive(Debug, Deserialize)]
struct ResolveResponse {
    address: String,
}

pub struct HLNamesClient {
    api_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl HLNamesClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            api_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub fn is_valid_name(&self, name: &str) -> bool {
        let labels = name.split('.').collect::<Vec<&str>>();
        if labels.is_empty() {
            return false;
        }

        !labels.iter().any(|label| label.is_empty())
    }

    async fn resolve_name(&self, name: &str) -> anyhow::Result<String> {
        let url = format!("{}/resolve/address/{}", self.api_url, name);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(API_KEY_HEADER, HeaderValue::from_str(&self.api_key)?);

        let response = self.client.get(url).headers(headers).send().await?;

        if response.status().is_success() {
            let body = response.json::<ResolveResponse>().await?;
            Ok(body.address)
        } else {
            Err(anyhow::anyhow!("Failed to resolve hlnames: {}", response.status()))
        }
    }
}

#[async_trait]
impl NameClient for HLNamesClient {
    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Hyperliquid]
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        if !self.chains().contains(&chain) {
            return Err(format!("Unsupported chain: {}", chain).into());
        }

        if !self.is_valid_name(name) {
            return Err(format!("Invalid name: {}", name).into());
        }

        self.resolve_name(name).await.map_err(|e| e.into())
    }

    fn provider(&self) -> NameProvider {
        NameProvider::Hyperliquid
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["hl"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_name() {
        let client = HLNamesClient::new("".to_string(), "".to_string());

        assert!(client.is_valid_name("test.hl"));
        assert!(client.is_valid_name("a.test.hl"));
        assert!(client.is_valid_name("a.b.test.hl"));

        assert!(!client.is_valid_name("test..hl"));
        assert!(!client.is_valid_name("test.hl."));
        assert!(!client.is_valid_name("test.hl.."));
        assert!(!client.is_valid_name("test.hl..hl"));
    }
}
