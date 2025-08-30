use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use chain_traits::{ChainBalances, ChainProvider, ChainStaking, ChainState, ChainToken, ChainTransactions};
use primitives::{Asset, AssetBalance, Chain, StakeValidator, Transaction};

pub struct GenericProvider<T> {
    client: T,
}

impl<T> GenericProvider<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<T> ChainBlockProvider for GenericProvider<T>
where
    T: ChainState + ChainTransactions + ChainProvider + Send + Sync,
{
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_block_latest_number().await? as i64)
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.client.get_transactions_by_block(block as u64).await
    }
}

#[async_trait]
impl<T> ChainTokenDataProvider for GenericProvider<T>
where
    T: ChainToken + ChainProvider + Send + Sync,
{
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl<T> ChainAssetsProvider for GenericProvider<T>
where
    T: ChainBalances + ChainProvider + Send + Sync,
{
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.client.get_assets_balances(address).await
    }
}

#[async_trait]
impl<T> ChainTransactionsProvider for GenericProvider<T>
where
    T: ChainTransactions + ChainProvider + Send + Sync,
{
    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.client.get_transactions_by_address(address, limit).await
    }
}

#[async_trait]
impl<T> ChainStakeProvider for GenericProvider<T>
where
    T: ChainStaking + ChainProvider + Send + Sync,
{
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
