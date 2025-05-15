use std::error::Error;

use chrono::Utc;
use jsonrpsee::{
    core::{client::ClientT, params::ObjectParams},
    http_client::{HttpClient, HttpClientBuilder},
};
use primitives::{Asset, Chain, TransactionState, TransactionType};

use super::model::{Action, Block, BlockHeader, Chunk};

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
        let block: Block = self.client.request("block", params).await?;
        Ok(block)
    }

    pub async fn get_block(&self, block: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("block_id", block)?;
        let block: Block = self.client.request("block", params).await?;
        Ok(block)
    }

    pub async fn get_chunk(&self, block: i64, shard_id: i64) -> Result<Chunk, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("block_id", block)?;
        params.insert("shard_id", shard_id)?;
        let chunk: Chunk = self.client.request("chunk", params).await?;
        Ok(chunk)
    }

    pub fn map_transaction(&self, header: BlockHeader, transaction: super::model::Transaction) -> Option<primitives::Transaction> {
        if transaction.actions.len() == 1 || transaction.actions.len() == 2 {
            match &transaction.actions.last()? {
                Action::Transfer { deposit } => {
                    let asset_id = Chain::Near.as_asset_id();
                    let transaction = primitives::Transaction::new(
                        transaction.hash,
                        asset_id.clone(),
                        transaction.signer_id,
                        transaction.receiver_id,
                        None,
                        TransactionType::Transfer,
                        TransactionState::Confirmed,
                        header.height.to_string(),
                        transaction.nonce.to_string(),
                        "830000000000000000000".to_string(),
                        asset_id,
                        deposit.clone(),
                        None,
                        None,
                        Utc::now(),
                    );
                    return Some(transaction);
                }
                Action::CreateAccount | Action::Other(_) => return None,
            }
        }
        None
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
