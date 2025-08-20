use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use chain_traits::ChainBalances;
use gem_client::Client;
use primitives::{chain::Chain, Asset, AssetBalance, Transaction};

use gem_tron::rpc::client::TronClient;
// use gem_tron::rpc::TronMapper; // Temporarily disabled

pub struct TronProvider<C: Client> {
    client: TronClient<C>,
}

impl<C: Client> TronProvider<C> {
    pub fn new(client: TronClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for TronProvider<C> {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, _block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        // Temporarily disabled until TronMapper is available
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for TronProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl<C: Client> ChainBalances for TronProvider<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        self.client.get_balance_coin(address).await
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        self.client.get_balance_tokens(address, token_ids).await
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        self.client.get_balance_staking(address).await
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for TronProvider<C> {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        // Tron doesn't have a bulk assets API - individual asset balances are fetched through ChainBalances trait
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for TronProvider<C> {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        // Transactions are temporarily disabled until TronGrid integration is complete
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for TronProvider<C> {
    async fn get_validators(&self) -> Result<Vec<primitives::StakeValidator>, Box<dyn Error + Send + Sync>> {
        // Temporarily disabled until TronMapper is available
        Ok(vec![])
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        // Temporarily disabled until TronMapper is available
        Ok(0.0)
    }
}

// Tron Grid implementations temporarily disabled
