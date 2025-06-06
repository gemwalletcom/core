use std::error::Error;

use primitives::chain::Chain;
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, Blocks, Data};

pub struct CardanoClient {
    chain: Chain,
    client: ClientWithMiddleware,
    url: String,
}

impl CardanoClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self {
            chain: Chain::Cardano,
            client,
            url,
        }
    }

    pub async fn get_tip_number(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": "{ cardano { tip { number } } }"
        });
        let response: serde_json::Value = self.client.post(self.url.as_str()).json(&json).send().await?.json().await?;
        response["data"]["cardano"]["tip"]["number"]
            .as_i64()
            .ok_or_else(|| "Could not fetch tip number".into())
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": "query GetBlockByNumber($blockNumber: Int!) { blocks(where: { number: { _eq: $blockNumber } }) { number hash forgedAt transactions { hash inputs { address value } outputs { address value } fee } } }",
            "variables": {
                "blockNumber": block_number
            },
            "operationName": "GetBlockByNumber"
        });
        self.client
            .post(self.url.as_str())
            .json(&json)
            .send()
            .await?
            .json::<Data<Blocks>>()
            .await?
            .data
            .blocks
            .first()
            .cloned()
            .ok_or_else(|| "Block not found".into())
    }
}

impl CardanoClient {
    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.get_tip_number().await
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<primitives::Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
