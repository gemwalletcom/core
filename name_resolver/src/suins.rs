use primitives::chain::Chain;
use async_trait::async_trait;
use std::error::Error;
use ethers::providers::{JsonRpcClient, Http, RetryClientBuilder, RetryClient};
use primitives::name::{NameRecord, NameProvider};
use crate::client::NameClient;

pub struct SuinsClient {
    client: RetryClient<Http>,
}

impl SuinsClient {
    pub fn new(api_url: String) -> Self {
        let provider = Http::new(reqwest::Url::parse(api_url.as_str()).unwrap());
        let client = RetryClientBuilder::default()
            .build(provider, Box::<ethers::providers::HttpRateLimitRetryPolicy>::default());

        Self {
            client,
        }
    }
}

#[async_trait]
impl NameClient for SuinsClient {
    
    fn provider() -> NameProvider {
        NameProvider::Suins
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let address: String = JsonRpcClient::request(&self.client, "suix_resolveNameServiceAddress", vec![serde_json::json!(name)]).await?;
        //TODO: Fix later to Self::provider()
        Ok(NameRecord { name: name.to_string(), chain, address, provider: NameProvider::Ens })
    }

    fn domains() -> Vec<&'static str> {
        vec![
            "sui",
        ]
    }

    fn chains() -> Vec<Chain> {
        vec![
            Chain::Sui,
        ]
    }
}
