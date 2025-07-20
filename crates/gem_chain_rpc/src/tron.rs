use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use gem_tron::rpc::trongrid::client::TronGridClient;
use primitives::{chain::Chain, Asset};
use primitives::{AssetBalance, Transaction};

use gem_tron::rpc::trongrid::mapper::TronGridMapper;
use gem_tron::rpc::TronClient;
use gem_tron::rpc::TronMapper;

pub struct TronProvider {
    client: TronClient,
    assets_provider: Box<dyn ChainAssetsProvider>,
    transactions_provider: Box<dyn ChainTransactionsProvider>,
}

impl TronProvider {
    pub fn new(client: TronClient, assets_provider: Box<dyn ChainAssetsProvider>, transactions_provider: Box<dyn ChainTransactionsProvider>) -> Self {
        Self {
            client,
            assets_provider,
            transactions_provider,
        }
    }
}

#[async_trait]
impl ChainBlockProvider for TronProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block_tranactions(block_number).await?;
        let reciepts = self.client.get_block_tranactions_reciepts(block_number).await?;

        Ok(TronMapper::map_transactions(self.get_chain(), block, reciepts))
    }
}

#[async_trait]
impl ChainTokenDataProvider for TronProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl ChainAssetsProvider for TronProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.assets_provider.get_assets_balances(address).await
    }
}

#[async_trait]
impl ChainTransactionsProvider for TronProvider {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.transactions_provider.get_transactions_by_address(address).await
    }
}

#[async_trait]
impl ChainStakeProvider for TronProvider {
    // Default implementation returns empty vector
}

// Tron Grid
#[async_trait]
impl ChainAssetsProvider for TronGridClient {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let accounts = self.get_accounts_by_address(&address).await?.data;
        if let Some(account) = accounts.first() {
            Ok(TronGridMapper::map_asset_balances(account.clone()))
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait]
impl ChainTransactionsProvider for TronGridClient {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let native_transactions = self.get_transactions_by_address(&address, 16).await?.data;
        let token_transactions = self.get_token_transactions(&address, 16).await?;

        let transactions = native_transactions
            .clone()
            .into_iter()
            .chain(token_transactions)
            .collect::<Vec<gem_tron::rpc::model::Transaction>>();

        if transactions.is_empty() {
            return Ok(vec![]);
        }
        let transaction_ids = transactions.iter().map(|x| x.tx_id.clone()).collect::<Vec<String>>();
        let reciepts = self.get_transactions_reciepts(transaction_ids).await?;

        Ok(TronGridMapper::map_transactions(transactions.clone(), reciepts))
    }
}
