use async_trait::async_trait;
use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use primitives::NameProvider;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    status_code: i64,
    name: String,
    address: String,
}

pub struct BNSClient {
    api_url: String,
    client: Client,
}

impl BNSClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::new();
        Self { api_url, client }
    }
}

#[async_trait]
impl NameClient for BNSClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Bns
    }

    async fn resolve(
        &self,
        name: &str,
        _chain: Chain,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/names/{}", self.api_url, name);
        let response: Response = self.client.get(&url).send().await?.json().await?;
        Ok(response.address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["base"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Base]
    }
}

mod tests {
    use super::*;
    use tokio_test::block_on;

    #[ignore]
    #[test]
    fn test_resolve() {
        // this test is ignored from UT casue it connects to the real network
        block_on(async {
            let client = super::BNSClient::new(String::from("https://resolver-api.basename.app"));
            let addres = client.resolve("hello.base", primitives::Chain::Base).await;
            assert_eq!(
                addres.unwrap(),
                "0x4fb3f133951bF1B2d52fF6CEab2c703fbB6E98cC"
            )
        });
    }
}
