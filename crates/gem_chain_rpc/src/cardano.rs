use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::AssetBalance;
use primitives::{chain::Chain, Asset};

use gem_cardano::rpc::CardanoClient;
use gem_cardano::rpc::CardanoMapper;

pub struct CardanoProvider {
    client: CardanoClient,
}

impl CardanoProvider {
    pub fn new(client: CardanoClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for CardanoProvider {
    fn get_chain(&self) -> Chain {
        Chain::Cardano
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_tip_number().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?;
        let transactions = block
            .transactions
            .clone()
            .into_iter()
            .flat_map(|x| CardanoMapper::map_transaction(self.client.get_chain(), &block, &x))
            .collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for CardanoProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        // Forward to the client implementation
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl ChainAssetsProvider for CardanoProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
