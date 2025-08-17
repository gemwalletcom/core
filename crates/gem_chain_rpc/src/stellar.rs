use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset};
use primitives::{AssetBalance, Transaction};

use gem_client::Client;
use gem_stellar::rpc::client::StellarClient;
use gem_stellar::rpc::mapper::StellarMapper;

pub struct StellarProvider<C: Client> {
    client: StellarClient<C>,
}

impl<C: Client> StellarProvider<C> {
    pub fn new(client: StellarClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for StellarProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Stellar
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_node_status().await?.ingest_latest_ledger as i64)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_block_payments_all(block_number).await?;
        Ok(StellarMapper::map_transactions(self.get_chain(), transactions))
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for StellarProvider<C> {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Err("Chain does not support tokens".into())
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for StellarProvider<C> {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let account = self.client.get_stellar_account(&address).await?;
        Ok(StellarMapper::map_balances(self.get_chain(), account))
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for StellarProvider<C> {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let payments = self.client.get_account_payments(address).await?;
        Ok(StellarMapper::map_transactions(self.get_chain(), payments))
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for StellarProvider<C> {}
