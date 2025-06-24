use std::error::Error;

use jsonrpsee::{
    core::{client::ClientT, params::ObjectParams},
    http_client::{HttpClient, HttpClientBuilder},
};
use primitives::{Asset, Chain};

use super::model::{Block, Chunk};

pub struct NearClient {
    client: HttpClient,
}

impl NearClient {
    pub fn new(url: String) -> Self {
        let client = HttpClientBuilder::default()
            .max_response_size(256 * 1024 * 1024) // 256MB
            .build(url)
            .unwrap();

        Self { client }
    }

    pub async fn get_final_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("finality", "final")?;
        Ok(self.client.request("block", params).await?)
    }

    pub async fn get_block(&self, block: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("block_id", block)?;
        Ok(self.client.request("block", params).await?)
    }

    pub async fn get_chunk(&self, block: i64, shard_id: i64) -> Result<Chunk, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("block_id", block)?;
        params.insert("shard_id", shard_id)?;
        Ok(self.client.request("chunk", params).await?)
    }
}

impl NearClient {
    pub fn get_chain(&self) -> Chain {
        Chain::Near
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
