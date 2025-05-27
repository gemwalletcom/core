use crate::client::NameClient;
use async_trait::async_trait;
use primitives::name::NameProvider;
use primitives::Chain;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde::Deserialize;
use std::env;
use std::error::Error;

const HLNAMES_API_URL_BASE: &str = "https://api.hlnames.xyz/resolve/address/";
const API_KEY_VALUE: &str = "CPEPKMI-HUSUX6I-SE2DHEA-YYWFG5Y";

#[derive(Debug, Deserialize)]
struct ResolveResponse {
    address: String,
}

pub struct HLNamesClient {
    api_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl Default for HLNamesClient {
    fn default() -> Self {
        let api_key = env::var("HLNAMES_API_KEY").unwrap_or_else(|_| API_KEY_VALUE.to_string());
        Self::new(HLNAMES_API_URL_BASE.to_string(), api_key)
    }
}

impl HLNamesClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            api_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn resolve_name(&self, name: &str) -> anyhow::Result<String> {
        let url = format!("{}{}", self.api_url, name);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key)?);

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
        self.resolve_name(name).await.map_err(|e| e.into())
    }

    fn provider(&self) -> NameProvider {
        NameProvider::Hyperliquid
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["hl"]
    }
}
