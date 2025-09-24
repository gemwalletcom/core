use std::error::Error;

use crate::models::{Block, Transactions};

#[cfg(feature = "rpc")]
use gem_client::Client;

#[derive(Clone, Debug)]
pub struct AlgorandClientIndexer<C: Client> {
    pub client: C,
}

impl<C: Client> AlgorandClientIndexer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_account_transactions(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/accounts/{}/transactions", address)).await?)
    }

    pub async fn get_block(&self, block_number: u64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/blocks/{}", block_number)).await?)
    }
}
