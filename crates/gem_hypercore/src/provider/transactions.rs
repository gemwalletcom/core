use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.transaction_broadcast(data).await
    }

    async fn get_transaction_status(&self, hash: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(hash)
    }
}
