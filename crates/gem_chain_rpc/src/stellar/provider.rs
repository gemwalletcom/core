use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset};

use super::client::StellarClient;
use super::mapper::StellarMapper;

pub struct StellarProvider {
    client: StellarClient,
}

impl StellarProvider {
    pub fn new(client: StellarClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for StellarProvider {
    fn get_chain(&self) -> Chain {
        Chain::Stellar
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_node_status().await.map(|status| status.history_latest_ledger)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?;
        let transactions = self
            .client
            .get_block_payments_all(block_number)
            .await?
            .iter()
            .flat_map(|x| StellarMapper::map_transaction(self.get_chain(), block.clone(), x.clone()))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for StellarProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}
