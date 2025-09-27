use async_trait::async_trait;
use chain_traits::ChainTransactions;
use primitives::BroadcastOptions;
use std::error::Error;

use gem_client::Client;

use crate::{provider::transactions_mapper::map_transaction_broadcast, rpc::client::HyperCoreClient};

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.exchange(serde_json::from_str(&data)?).await?;
        map_transaction_broadcast(response, data)
    }
}
