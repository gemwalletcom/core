use async_trait::async_trait;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use primitives::{chain::Chain, name::NameProvider};

#[derive(Debug, Deserialize, Serialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub handle_to_address: Option<String>,
}

pub struct LensClient {
    api_url: String,
    client: Client,
}

impl LensClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::new();
        Self { api_url, client }
    }
}

#[async_trait]
impl NameClient for LensClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Lens
    }

    async fn resolve(
        &self,
        name: &str,
        _chain: Chain,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut parts = name.split('.').collect::<Vec<&str>>();
        parts.reverse();
        let handle = parts.join("/");

        let query = format!(
            "query {{handleToAddress(request: {{handle: \"{}\" }} )}}",
            handle
        );
        let query = serde_json::json!({
            "query": query,
        });

        let address = self
            .client
            .post(&self.api_url)
            .json(&query)
            .send()
            .await?
            .json::<Data<Record>>()
            .await?
            .data
            .handle_to_address;

        address.ok_or("address not found".into())
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["lens"]
    }

    fn chains(&self) -> Vec<Chain> {
        // Add all evm chains?
        vec![Chain::Ethereum, Chain::Polygon]
    }
}
