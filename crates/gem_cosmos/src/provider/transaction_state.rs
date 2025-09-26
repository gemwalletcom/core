use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::CosmosClient;

use super::transaction_state_mapper::map_transaction_status;

#[async_trait]
impl<C: Client> ChainTransactionState for CosmosClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction_status(self.get_transaction(request.id).await?))
    }
}
