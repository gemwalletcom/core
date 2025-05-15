use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset};

use super::client::TronClient;
use super::mapper::TronMapper;

pub struct TronProvider {
    client: TronClient,
}

impl TronProvider {
    pub fn new(client: TronClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for TronProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block_tranactions(block_number).await?;
        let transactions = block.transactions.unwrap_or_default();
        let reciepts = self.client.get_block_tranactions_reciepts(block_number).await?;

        let transactions = transactions
            .into_iter()
            .zip(reciepts.iter())
            .filter_map(|(transaction, receipt)| TronMapper::map_transaction(self.get_chain(), transaction, receipt.clone()))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for TronProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}
