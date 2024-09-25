use async_trait::async_trait;
use primitives::chain::Chain;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use primitives::NameProvider;

#[derive(Debug, Deserialize, Serialize)]
pub struct RealmResolveRecord {
    pub owner: String,
}

pub struct RealmClient {
    client: Client,
}

impl RealmClient {
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }
}

#[async_trait]
impl NameClient for RealmClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Realm
    }

    async fn resolve(
        &self,
        name: &str,
        _chain: Chain,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {

        let realm_name = &name[1..];
        let url = format!("https://realm.fan/api/gemwallet/{}", realm_name);
        let record: RealmResolveRecord = self.client.get(&url).send().await?.json().await?;
        let address = record.owner;

        Ok(address)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["+"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Bitcoin]
    }
}
