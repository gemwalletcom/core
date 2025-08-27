use crate::client::NameClient;
use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::NameProvider;
use std::error::Error;

use anyhow::{anyhow, Result};
use gem_client::ReqwestClient;
use gem_jsonrpc::{types::JsonRpcError, JsonRpcClient};

pub struct SuinsClient {
    client: JsonRpcClient<ReqwestClient>,
}

impl SuinsClient {
    pub fn new(api_url: String) -> Self {
        let client = JsonRpcClient::new(ReqwestClient::new(api_url, reqwest::Client::new()));
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
        let address: String = self
            .client
            .call("suix_resolveNameServiceAddress", params)
            .await
            .map_err(|e: JsonRpcError| anyhow!(e))?;
        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sui"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Sui]
    }
}
