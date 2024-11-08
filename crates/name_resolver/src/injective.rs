use crate::client::NameClient;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use primitives::{Chain, NameProvider};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct InjectiveNameClient {
    url: String,
    client: Client,
}

// request
#[derive(Debug, Deserialize, Serialize)]
pub struct Resolver {
    resolver: ResolverNode,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverNode {
    node: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverAddress {
    address: ResolverNode,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverDataResponse {
    data: ResolverData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverData {
    address: String,
}

//const REGISTER_ADDRESS: &str = "inj1hm8vs8sr2h9nk0x66vctfs528wrp6k3gtgg275";
const RESOLVER_ADDRESS: &str = "inj1x9m0hceug9qylcyrrtwqtytslv2jrph433thgu";

impl InjectiveNameClient {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }
}

#[async_trait]
impl NameClient for InjectiveNameClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Injective
    }

    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let hash = crate::ens_provider::namehash::namehash(name);
        let resolve = ResolverAddress {
            address: ResolverNode { node: hash },
        };

        let string = serde_json::to_string(&resolve)?;
        let encoded = general_purpose::STANDARD.encode(string).to_string();

        let url = format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}", self.url, RESOLVER_ADDRESS, encoded);

        let response = self.client.get(&url).send().await?.json::<ResolverDataResponse>().await?;

        Ok(response.data.address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["inj"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Injective]
    }
}
