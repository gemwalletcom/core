use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};
use std::error::Error;

use gem_client::Client;
use primitives::BroadcastOptions;

use crate::{
    provider::{BroadcastProvider, transaction_broadcast_mapper::map_transaction_broadcast_response_from_str},
    rpc::client::SolanaClient,
};

#[async_trait]
impl<C: Client + Clone> ChainTransactionBroadcast for SolanaClient<C> {
    async fn transaction_broadcast(&self, data: String, options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.send_transaction(data, Some(options.skip_preflight)).await?;
        Ok(response)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, response: &str) -> Option<String> {
        map_transaction_broadcast_response_from_str(response).ok()
    }
}
