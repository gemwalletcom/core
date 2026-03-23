use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};
use std::error::Error;

use gem_client::Client;
use primitives::BroadcastOptions;

use crate::{
    provider::{
        BroadcastProvider,
        transactions_mapper::{map_transaction_broadcast, map_transaction_broadcast_from_str},
    },
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransactionBroadcast for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let request = serde_json::from_str(&data)?;
        let response = self.exchange(request).await?;
        map_transaction_broadcast(response, &data)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, response: &str) -> Option<String> {
        map_transaction_broadcast_from_str(response).ok()
    }
}
