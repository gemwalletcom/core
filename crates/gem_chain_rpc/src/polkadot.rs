use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset};
use primitives::{AssetBalance, Transaction};

use gem_polkadot::rpc::client::PolkadotClient;
use gem_polkadot::rpc::mapper::PolkadotMapper;

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
        Ok(self.client.get_block_header("head").await?.number as i64)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?;
        Ok(PolkadotMapper::map_transactions(self.get_chain(), block.clone()))
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

#[async_trait]
impl ChainTransactionsProvider for PolkadotProvider {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl ChainStakeProvider for PolkadotProvider { }
