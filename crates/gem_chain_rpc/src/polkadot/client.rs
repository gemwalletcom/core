use std::error::Error;

use primitives::{Asset, Chain};
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, BlockHeader};

pub struct PolkadotClient {
    url: String,
    client: ClientWithMiddleware,
}

impl PolkadotClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_block_header(&self, block: &str) -> Result<BlockHeader, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/blocks/{}/header", self.url, block);
        Ok(self.client.get(url).send().await?.json::<BlockHeader>().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/blocks/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<Block>().await?)
    }

    // Transaction mapping methods moved to PolkadotMapper

    pub fn get_chain(&self) -> Chain {
        Chain::Polkadot
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
