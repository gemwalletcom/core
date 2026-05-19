use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainTransactionState for BitcoinClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&request.id).await?;
        let status = if transaction.is_confirmed() {
            TransactionState::Confirmed
        } else {
            TransactionState::Pending
        };
        Ok(TransactionUpdate::new_state(status))
    }
}
