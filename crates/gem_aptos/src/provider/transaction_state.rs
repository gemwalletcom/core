use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{provider::transaction_state_mapper::map_transaction_status, rpc::client::AptosClient};

#[async_trait]
impl<C: Client> ChainTransactionState for AptosClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction_status(&self.get_transaction_by_hash(&request.id).await?))
    }
}
