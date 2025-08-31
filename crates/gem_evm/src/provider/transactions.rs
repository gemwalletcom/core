use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactions;
use primitives::{BroadcastOptions, TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::EthereumClient;
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for EthereumClient<C> {
    async fn transaction_broadcast(&self, _data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        unimplemented!("transaction_broadcast")
    }

    async fn get_transaction_status(&self, _request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        unimplemented!("get_transaction_status")
    }
}
