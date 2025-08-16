use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::AssetBalance;
use primitives::{chain::Chain, Asset, Transaction};

use gem_near::rpc::NearClient;
use gem_near::rpc::NearMapper;

pub struct NearProvider {
    client: NearClient,
}

impl NearProvider {
    pub fn new(client: NearClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for NearProvider {
    fn get_chain(&self) -> Chain {
        Chain::Near
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_final_block().await?.header.height)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await;
        match block {
            Ok(block) => {
                let chunks =
                    futures::future::try_join_all(block.chunks.into_iter().map(|chunk| self.client.get_chunk(block.header.height, chunk.shard_id))).await?;

                let transactions = chunks
                    .into_iter()
                    .flat_map(|x| {
                        x.transactions
                            .into_iter()
                            .flat_map(|x| NearMapper::map_transaction(self.get_chain(), block.header.clone(), x))
                    })
                    .collect();
                Ok(transactions)
            }
            Err(_) => {
                // Skipping block for now, same as in client implementation
                Ok(vec![])
            }
        }
    }
}

#[async_trait]
impl ChainTokenDataProvider for NearProvider {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        // The client's get_token_data is just an unimplemented stub for now
        unimplemented!()
    }
}

#[async_trait]
impl ChainAssetsProvider for NearProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl ChainTransactionsProvider for NearProvider {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl ChainStakeProvider for NearProvider { }
