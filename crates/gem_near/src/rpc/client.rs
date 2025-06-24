use std::error::Error;

use gem_jsonrpc::JsonRpcClient;
use primitives::{Asset, Chain};
use serde_json::json;

use super::model::{Block, Chunk};

pub struct NearClient {
    client: JsonRpcClient,
}

impl NearClient {
    pub fn new(client: JsonRpcClient) -> Self {
        Self { client }
    }

    pub async fn get_final_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "finality": "final"
        });
        Ok(self.client.call("block", params).await?)
    }

    pub async fn get_block(&self, block: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "block_id": block
        });
        Ok(self.client.call("block", params).await?)
    }

    pub async fn get_chunk(&self, block: i64, shard_id: i64) -> Result<Chunk, Box<dyn Error + Send + Sync>> {
        let params = json!({
            "block_id": block,
            "shard_id": shard_id
        });
        Ok(self.client.call("chunk", params).await?)
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
