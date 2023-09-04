use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;

use super::model::{Block, NodeInfo};

pub struct BNBChainClient {
    url: String,
    api_url: String,
    
    client: reqwest::Client,
}

impl BNBChainClient {
    pub fn new(url: String, api_url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            url,
            api_url,
            client,
        }
    }
}

#[async_trait]
impl ChainProvider for BNBChainClient {
    async fn get_latest_block(&self) -> Result<i32, Box<dyn Error>> {
        let url = format!("{}/api/v1/node-info", self.url);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<NodeInfo>()
            .await?;

        return Ok(response.sync_info.latest_block_height);
    }

    async fn get_transactions(&self, block: i32) -> Result<Vec<i32>, Box<dyn Error>> {
        let url = format!("{}/bc/api/v1/blocks/{}/txs", self.api_url, block);

        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<Block>()
            .await?;

        let _transactions = response.txs;

        return Ok(vec![]);
    }
}