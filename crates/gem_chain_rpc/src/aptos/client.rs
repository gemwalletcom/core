use std::error::Error;

use primitives::chain::Chain;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

use gem_aptos::model::{Block, Ledger, Resource};

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
        let response = self.client.get(url).send().await?.json::<Ledger>().await?;
        Ok(response)
    }

    pub async fn get_block_transactions(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/blocks/by_height/{}?with_transactions=true", self.url, block_number);
        let response = self.client.get(url).send().await?.json::<Block>().await?;

        Ok(response)
    }

    pub async fn get_resource<T: Serialize + for<'a> Deserialize<'a>>(
        &self,
        address: String,
        resource: String,
    ) -> Result<Resource<T>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/accounts/{}/resource/{}", self.url, address, resource);
        Ok(self.client.get(url).send().await?.json::<Resource<T>>().await?)
    }
}
