use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainTransactionState for CardanoClient<C> {
    async fn get_transaction_status(&self, _request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Ok(TransactionUpdate::new_state(TransactionState::Confirmed))
    }
}
