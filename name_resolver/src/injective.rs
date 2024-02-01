use crate::client::NameClient;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use primitives::{
    chain::Chain,
    name::{NameProvider, NameRecord},
};
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
// response

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverResponse {
    data: ResolverData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolverData {
    resolver: String,
}

const RESOLVE_ADDRESS: &str = "inj1hm8vs8sr2h9nk0x66vctfs528wrp6k3gtgg275";

impl InjectiveNameClient {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }
}

#[async_trait]
impl NameClient for InjectiveNameClient {
    fn provider() -> NameProvider {
        NameProvider::Injective
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<NameRecord, Box<dyn Error>> {
        let hash = crate::ens_provider::namehash::namehash(name);
        let resolve = Resolver {
            resolver: ResolverNode { node: hash },
        };

        let string = serde_json::to_string(&resolve)?;
        let encoded = general_purpose::STANDARD.encode(string).to_string();

        let url = format!(
            "{}/cosmwasm/wasm/v1/contract/{}/smart/{}",
            self.url, RESOLVE_ADDRESS, encoded
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<ResolverResponse>()
            .await?;

        //TODO: Use provider: self.provider
        Ok(NameRecord {
            name: name.to_string(),
            chain,
            address: response.data.resolver,
            provider: NameProvider::SpaceId,
        })
    }

    fn domains() -> Vec<&'static str> {
        vec!["inj"]
    }

    fn chains() -> Vec<Chain> {
        vec![Chain::Injective]
    }
}
