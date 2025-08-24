use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use chain_traits::ChainStaking;
use futures::future;
use primitives::Transaction;
use primitives::{Asset, AssetBalance, Chain, StakeValidator};

use gem_client::Client;
use gem_cosmos::rpc::CosmosClient;
use gem_cosmos::rpc::CosmosMapper;

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
        Ok(self.client.get_block("latest").await?.block.header.height.parse()?)
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_block(block.to_string().as_str()).await?;
        let transaction_ids = response
            .block
            .data
            .txs
            .clone()
            .into_iter()
            .flat_map(CosmosMapper::map_transaction_decode)
            .collect::<Vec<_>>();
        let receipts = future::try_join_all(transaction_ids.into_iter().map(|x| self.client.get_transaction(x))).await?;

        Ok(CosmosMapper::map_transactions(self.get_chain(), receipts))
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainTokenDataProvider for CosmosProvider<C> {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainAssetsProvider for CosmosProvider<C> {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client + Send + Sync> ChainTransactionsProvider for CosmosProvider<C> {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_transactions_by_address(&address, 20).await?;
        Ok(CosmosMapper::map_transactions(self.get_chain(), transactions))
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
            .map(|validator| StakeValidator::new(validator.id, validator.name))
            .collect())
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_staking_apy().await?.unwrap_or(0.0))
    }
}
