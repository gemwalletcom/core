use crate::client::NameClient;
use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::NameProvider;
use std::error::Error;

use alloy_rpc_client::RpcClient;
use alloy_transport_http::Http;
use url::Url;
use anyhow::{anyhow, Result};

pub struct SuinsClient {
    client: RpcClient,
}

impl SuinsClient {
    pub fn new(api_url: String) -> Self {
        let url = Url::parse(&api_url).expect("Invalid Suins API URL");
        let http_transport = Http::new(url);
        let client = RpcClient::new(http_transport, true);
        Self { client }
    }
}

#[async_trait]
impl NameClient for SuinsClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Suins
    }

    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let params = vec![serde_json::json!(name)];
        let address: String = self.client.request("suix_resolveNameServiceAddress", params).await.map_err(|e| anyhow!(e))?;
        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sui"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Sui]
    }
}
