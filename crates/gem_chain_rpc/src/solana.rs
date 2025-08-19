use gem_client::Client;

pub struct SolanaProvider<C: Client + Clone> {
    client: SolanaClient<C>,
}

impl<C: Client + Clone> SolanaProvider<C> {
    pub fn new(client: SolanaClient<C>) -> Self {
        Self { client }
    }
}

use async_trait::async_trait;
use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use gem_solana::rpc::client::SolanaClient;
use primitives::{chain::Chain, Transaction};

#[async_trait]
impl<C: Client + Clone> ChainBlockProvider for SolanaProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Solana
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_slot().await? as i64)
    }

    async fn get_transactions(&self, _block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactionsProvider for SolanaProvider<C> {}

#[async_trait]
impl<C: Client + Clone> ChainStakeProvider for SolanaProvider<C> {}

#[async_trait]
impl<C: Client + Clone> ChainAssetsProvider for SolanaProvider<C> {}

#[async_trait]
impl<C: Client + Clone> ChainTokenDataProvider for SolanaProvider<C> {}
