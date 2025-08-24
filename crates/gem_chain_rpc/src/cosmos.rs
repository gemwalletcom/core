use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use chain_traits::{ChainBalances, ChainStaking, ChainState, ChainToken, ChainTransactions};
use primitives::Transaction;
use primitives::{Asset, AssetBalance, Chain, StakeValidator};

use gem_client::Client;
use gem_cosmos::rpc::CosmosClient;

pub struct CosmosProvider<C: Client> {
    client: CosmosClient<C>,
}

impl<C: Client> CosmosProvider<C> {
    pub fn new(client: CosmosClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainBlockProvider for CosmosProvider<C> {
    fn get_chain(&self) -> Chain {
        self.client.get_chain().as_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_block_latest_number().await? as i64)
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.client.get_transactions_by_block(block as u64).await
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainTokenDataProvider for CosmosProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainAssetsProvider for CosmosProvider<C> {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.client.get_assets_balances(address).await
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainTransactionsProvider for CosmosProvider<C> {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.client.get_transactions_by_address(address).await
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainStakeProvider for CosmosProvider<C> {
    async fn get_validators(&self) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get_staking_validators(None)
            .await?
            .into_iter()
            .map(|x| StakeValidator::new(x.id, x.name))
            .collect())
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_staking_apy().await?.unwrap_or(0.0))
    }
}
