use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, Transaction};

use gem_client::Client;
use gem_ton::rpc::{TonClient, TonMapper};

pub struct TonProvider<C: Client> {
    client: TonClient<C>,
}

impl<C: Client> TonProvider<C> {
    pub fn new(client: TonClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for TonProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Ton
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_transactions(block_number.to_string()).await?.transactions;
        Ok(TonMapper::map_transactions(self.get_chain(), transactions))
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for TonProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for TonProvider<C> {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        // TON doesn't have a bulk assets API in the original implementation
        // Return empty balances for now - individual asset balances are fetched through ChainBalances trait
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for TonProvider<C> {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_transactions_by_address(address, 20).await?;
        Ok(TonMapper::map_transactions(self.get_chain(), transactions.transactions))
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for TonProvider<C> {}
