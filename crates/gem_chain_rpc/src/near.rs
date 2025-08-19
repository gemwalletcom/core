use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, Transaction};

use gem_client::Client;
use gem_near::rpc::client::NearClient;

pub struct NearProvider<C: Client + Clone> {
    client: NearClient<C>,
}

impl<C: Client + Clone> NearProvider<C> {
    pub fn new(client: NearClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client + Clone> ChainBlockProvider for NearProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Near
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_latest_block().await?.header.height as i64)
    }

    async fn get_transactions(&self, _block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTokenDataProvider for NearProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_token_data(token_id).await?)
    }
}

#[async_trait]
impl<C: Client + Clone> ChainAssetsProvider for NearProvider<C> {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let account = self.client.get_account(&address).await?;
        let asset_id = self.get_chain().as_asset_id();
        let balance = AssetBalance::new(asset_id, account.amount);
        Ok(vec![balance])
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactionsProvider for NearProvider<C> {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client + Clone> ChainStakeProvider for NearProvider<C> {}
