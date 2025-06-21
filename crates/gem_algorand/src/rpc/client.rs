use std::error::Error;

use reqwest_middleware::ClientWithMiddleware;

use crate::rpc::model::{Account, AssetResponse, BlockHeaders, Transactions};

use super::model::Block;

pub struct AlgorandClient {
    url: String,
    client: ClientWithMiddleware,
}

impl AlgorandClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_block_headers(&self) -> Result<BlockHeaders, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/block-headers", self.url);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/accounts/{}", self.url, address);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_asset(&self, asset_id: &str) -> Result<AssetResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/assets/{}", self.url, asset_id);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_account_transactions(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/accounts/{}/transactions", self.url, address);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blocks/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json().await?)
    }
}
