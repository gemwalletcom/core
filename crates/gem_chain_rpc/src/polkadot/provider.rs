use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::AssetBalance;
use primitives::{chain::Chain, Asset};

use super::client::PolkadotClient;
use super::mapper::PolkadotMapper;

pub struct PolkadotProvider {
    client: PolkadotClient,
}

impl PolkadotProvider {
    pub fn new(client: PolkadotClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for PolkadotProvider {
    fn get_chain(&self) -> Chain {
        Chain::Polkadot
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client
            .get_block_header("head")
            .await?
            .number
            .parse()
            .map_err(|_| "Failed to parse block number".into())
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?;

        let transactions = block
            .extrinsics
            .iter()
            .flat_map(|x| PolkadotMapper::map_transaction(self.get_chain(), block.clone(), x.clone()))
            .flatten()
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for PolkadotProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl ChainAssetsProvider for PolkadotProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
