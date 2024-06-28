use std::error::Error;

use async_trait::async_trait;
use chrono::Utc;
use jsonrpsee::{
    core::{client::ClientT, params::ObjectParams},
    http_client::{HttpClient, HttpClientBuilder},
};
use primitives::{Chain, Transaction, TransactionState, TransactionType};

use crate::ChainProvider;

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

    async fn get_final_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("finality", "final")?;
        let block: Block = self.client.request("block", params).await?;
        Ok(block)
    }

    async fn get_block(&self, block: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("block_id", block)?;
        let block: Block = self.client.request("block", params).await?;
        Ok(block)
    }

    async fn get_chunk(
        &self,
        block: i64,
        shard_id: i64,
    ) -> Result<Chunk, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("block_id", block)?;
        params.insert("shard_id", shard_id)?;
        let chunk: Chunk = self.client.request("chunk", params).await?;
        Ok(chunk)
    }

    fn map_transaction(
        &self,
        header: BlockHeader,
        transaction: super::model::Transaction,
    ) -> Option<primitives::Transaction> {
        if transaction.actions.len() == 1 || transaction.actions.len() == 2 {
            match &transaction.actions.last()? {
                Action::Transfer { deposit } => {
                    let asset_id = self.get_chain().as_asset_id();
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

#[async_trait]
impl ChainProvider for NearClient {
    fn get_chain(&self) -> Chain {
        Chain::Near
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.get_final_block().await?;
        Ok(block.header.height)
    }

    async fn get_transactions(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await;
        match block {
            Ok(block) => {
                let chunks = futures::future::try_join_all(
                    block
                        .chunks
                        .into_iter()
                        .map(|chunk| self.get_chunk(block.header.height, chunk.shard_id)),
                )
                .await?;

                let transactions = chunks
                    .into_iter()
                    .flat_map(|x| {
                        x.transactions
                            .into_iter()
                            .flat_map(|x| self.map_transaction(block.header.clone(), x))
                    })
                    .collect();
                Ok(transactions)
            }
            Err(_) => {
                //skipping block for now, need to check error for missing block
                // jsonrpsee::core::ClientError::Call(err) => {
                //     let errors = [-32000];
                //     if errors.contains(&err.code()) {
                //         unimplemented!("Block not found")
                //     } else {
                //         Err(Box::new(err))
                //     }
                // }
                // _ => Err(Box::new(err)),
                Ok(vec![])
            }
        }
    }
}
