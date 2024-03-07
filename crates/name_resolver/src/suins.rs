use crate::client::NameClient;
use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::NameProvider;
use std::error::Error;

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
};

pub struct SuinsClient {
    client: HttpClient,
}

impl SuinsClient {
    pub fn new(api_url: String) -> Self {
        let client = HttpClientBuilder::default().build(api_url).unwrap();
        Self { client }
    }
}

#[async_trait]
impl NameClient for SuinsClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Suins
    }

    async fn resolve(
        &self,
        name: &str,
        _chain: Chain,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let address = self
            .client
            .request(
                "suix_resolveNameServiceAddress",
                vec![serde_json::json!(name)],
            )
            .await?;
        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sui"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Sui]
    }
}
