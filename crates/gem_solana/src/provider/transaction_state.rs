use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactionState for SolanaClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&request.id).await?;

        if transaction.slot > 0 {
            if transaction.meta.err.is_some() {
                Ok(TransactionUpdate::new_state(TransactionState::Failed))
            } else {
                Ok(TransactionUpdate::new_state(TransactionState::Confirmed))
            }
        } else {
            Ok(TransactionUpdate::new_state(TransactionState::Pending))
        }
    }
}
