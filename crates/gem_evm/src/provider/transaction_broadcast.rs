use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionBroadcast;
use chain_traits::ChainTransactionDecode;
use primitives::BroadcastOptions;

use crate::{
    provider::{
        BroadcastProvider,
        transaction_broadcast_mapper::{map_transaction_broadcast_request, map_transaction_broadcast_response_from_str},
    },
    rpc::client::EthereumClient,
};
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionBroadcast for EthereumClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let data = map_transaction_broadcast_request(&data);
        let response = self.send_raw_transaction(&data).await?;
        Ok(response)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, response: &str) -> Option<String> {
        map_transaction_broadcast_response_from_str(response).ok()
    }
}
