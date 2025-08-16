use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::Transaction;
use primitives::{Asset, AssetBalance, Chain};

use gem_algorand::rpc::client::AlgorandClient;
use gem_algorand::rpc::AlgorandMapper;
use gem_client::Client;

pub struct AlgorandProvider<C: Client> {
    client: AlgorandClient<C>,
}

impl<C: Client> AlgorandProvider<C> {
    pub fn new(client: AlgorandClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for AlgorandProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Algorand
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_block_headers().await?.current_round)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?;
        Ok(AlgorandMapper::map_transactions(self.get_chain(), block.transactions))
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for AlgorandProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let asset = self.client.get_asset(&token_id).await?;
        Ok(AlgorandMapper::map_asset(asset.asset))
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for AlgorandProvider<C> {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let account = self.client.get_account(&address).await?;
        Ok(AlgorandMapper::map_assets_balance(account.assets))
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for AlgorandProvider<C> {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_account_transactions(&address).await?.transactions;
        Ok(AlgorandMapper::map_transactions(self.get_chain(), transactions))
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for AlgorandProvider<C> {}
