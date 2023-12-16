use crate::client::NameClient;
use async_trait::async_trait;
use primitives::chain::Chain;
use primitives::name::{NameProvider, NameRecord};
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
    fn provider() -> NameProvider {
        NameProvider::Suins
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let address = self
            .client
            .request(
                "suix_resolveNameServiceAddress",
                vec![serde_json::json!(name)],
            )
            .await?;
        Ok(NameRecord {
            name: name.to_string(),
            chain,
            address,
            provider: Self::provider(),
        })
    }

    fn domains() -> Vec<&'static str> {
        vec!["sui"]
    }

    fn chains() -> Vec<Chain> {
        vec![Chain::Sui]
    }
}
