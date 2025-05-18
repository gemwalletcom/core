use std::error::Error;

use primitives::chain::Chain;
use reqwest_middleware::ClientWithMiddleware;
use serde::{de::DeserializeOwned, Serialize};

use gem_aptos::model::{Block, Ledger, Resource, ResourceData};

pub type AccountResource<T> = Resource<T>;

#[derive(Clone)]
pub struct AptosClient {
    url: String,
    client: ClientWithMiddleware,
}

impl AptosClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Aptos
    }

    pub async fn get_ledger(&self) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/", self.url);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_block_transactions(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/blocks/by_height/{}?with_transactions=true", self.url, block_number);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_account_resource<T: Serialize + DeserializeOwned>(
        &self,
        address: String,
        resource: &str,
    ) -> Result<Option<AccountResource<T>>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/accounts/{}/resource/{}", self.url, address, resource);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    pub async fn get_account_resources(&self, address: &str) -> Result<Vec<Resource<ResourceData>>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/accounts/{}/resources", self.url, address);
        Ok(self.client.get(&url).send().await?.json().await?)
    }
}
