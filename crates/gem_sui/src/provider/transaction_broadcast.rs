use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};

use gem_client::Client;
use primitives::BroadcastOptions;

use crate::{
    provider::{
        BroadcastProvider,
        transaction_broadcast_mapper::{map_transaction_broadcast_request, map_transaction_broadcast_response, map_transaction_broadcast_response_from_str},
    },
    rpc::client::SuiClient,
};

#[async_trait]
impl<C: Client + Clone> ChainTransactionBroadcast for SuiClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let (transaction_data, signature) = map_transaction_broadcast_request(&data)?;
        let response = self.broadcast(transaction_data, signature).await?;
        map_transaction_broadcast_response(response)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, response: &str) -> Option<String> {
        map_transaction_broadcast_response_from_str(response).ok()
    }
}
