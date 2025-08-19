use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use futures::future;
use primitives::Transaction;
use primitives::{chain_cosmos::CosmosChain, Asset, AssetBalance, Chain, StakeValidator};

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
        let validators = self.client.get_validators().await?;
        Ok(CosmosMapper::map_validators(validators.validators))
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        match self.client.get_chain() {
            CosmosChain::Noble | CosmosChain::Thorchain => Ok(0.0),
            CosmosChain::Osmosis => Ok(14.0),
            CosmosChain::Celestia => Ok(15.0),
            CosmosChain::Sei => Ok(12.0),
            CosmosChain::Cosmos | CosmosChain::Injective => {
                let inflation = self.client.get_inflation().await?;
                let pool = self.client.get_staking_pool().await?;

                let bonded_tokens: f64 = pool.pool.bonded_tokens.parse()?;
                let total_tokens = bonded_tokens + pool.pool.not_bonded_tokens.parse::<f64>()?;
                let inflation_rate: f64 = inflation.inflation.parse()?;

                let staking_ratio = bonded_tokens / total_tokens;
                let apy = (inflation_rate / staking_ratio) * 100.0;

                Ok(apy)
            }
        }
    }
}
