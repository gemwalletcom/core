use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};

use crate::models::SolanaTransaction;
use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactionState for SolanaClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&request.id).await?;

        Ok(map_transaction_update(transaction.as_ref()))
    }
}

fn map_transaction_update(transaction: Option<&SolanaTransaction>) -> TransactionUpdate {
    let Some(transaction) = transaction else {
        return TransactionUpdate::new_state(TransactionState::Pending);
    };
    if transaction.meta.has_error() {
        TransactionUpdate::new_state(TransactionState::Reverted)
    } else {
        TransactionUpdate::new_state(TransactionState::Confirmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::JsonRpcResult;

    #[test]
    fn test_map_transaction_update_confirmed() {
        let confirmed = serde_json::from_str::<JsonRpcResult<Option<SolanaTransaction>>>(include_str!("../../testdata/transaction_state_transfer_sol.json"))
            .unwrap()
            .result;

        assert_eq!(map_transaction_update(confirmed.as_ref()), TransactionUpdate::new_state(TransactionState::Confirmed));
    }

    #[test]
    fn test_map_transaction_update_reverted() {
        let reverted = serde_json::from_str::<JsonRpcResult<Option<SolanaTransaction>>>(include_str!("../../testdata/transaction_state_reverted_program_account_not_found.json"))
            .unwrap()
            .result;

        assert_eq!(map_transaction_update(reverted.as_ref()), TransactionUpdate::new_state(TransactionState::Reverted));
    }

    #[test]
    fn test_map_transaction_update_pending_when_transaction_not_found() {
        let pending = serde_json::from_str::<JsonRpcResult<Option<SolanaTransaction>>>(include_str!("../../testdata/transaction_state_pending_not_found.json"))
            .unwrap()
            .result;

        assert_eq!(map_transaction_update(pending.as_ref()), TransactionUpdate::new_state(TransactionState::Pending));
    }
}
