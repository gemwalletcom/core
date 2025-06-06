use crate::client::NameClient;
use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::NameProvider;
use std::error::Error;

use alloy_rpc_client::{ClientBuilder, RpcClient};
use anyhow::{anyhow, Result};
use url::Url;

pub struct SuinsClient {
    client: RpcClient,
}

impl SuinsClient {
    pub fn new(api_url: String) -> Self {
        let url = Url::parse(&api_url).expect("Invalid Suins API URL");
        let client = ClientBuilder::default().http(url);
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
