use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction};

use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.broadcast_transaction(&data).await?.transaction.hash)
    }

    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_transactions_by_address(&self, _address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
