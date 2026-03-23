use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};
use std::error::Error;

use gem_client::Client;
use primitives::BroadcastOptions;

use crate::{
    provider::{BroadcastProvider, transaction_broadcast_mapper::map_transaction_broadcast_response_from_str, transactions_mapper::map_transaction_broadcast},
    rpc::client::XRPClient,
};

#[async_trait]
impl<C: Client + Clone> ChainTransactionBroadcast for XRPClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&response)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, response: &str) -> Option<String> {
        map_transaction_broadcast_response_from_str(response).ok()
    }
}
