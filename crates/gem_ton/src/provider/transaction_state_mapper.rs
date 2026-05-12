use std::error::Error;

use primitives::{TransactionChange, TransactionStateRequest, TransactionUpdate};

use crate::models::TraceResponse;
use crate::provider::transactions_mapper::map_transaction_state;

pub fn map_transaction_status(_request: TransactionStateRequest, traces: TraceResponse) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
    let transaction = traces.root_transaction().ok_or("Transaction not found")?;
    let state = if traces.has_actions() {
        traces.action_state().ok_or("Trace not found")?
    } else {
        map_transaction_state(transaction)
    };

    let fee = transaction.total_fees.clone();

    Ok(TransactionUpdate::new(state, vec![TransactionChange::NetworkFee(fee.into())]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MessageTransactions, TraceAction};
    use crate::provider::testkit::{FAILED_SWAP_MESSAGE_HASH, SUCCESS_SWAP_MESSAGE_HASH};
    use primitives::TransactionState;

    #[test]
    fn test_map_transaction_status_confirmed() {
        let request = TransactionStateRequest::new_id("hash".to_string());
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();
        let traces = TraceResponse::mock(transactions.transactions.first().unwrap().clone(), false, vec![]);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(!update.changes.is_empty());
    }

    #[test]
    fn test_ton_transaction_jetton_transfer_reverted() {
        let request = TransactionStateRequest::new_id("hash".to_string());
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_error_2.json")).unwrap();
        let traces = TraceResponse::mock(transactions.transactions.first().unwrap().clone(), false, vec![]);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Reverted);
        assert!(!update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_status_success_trace_action() {
        let request = TransactionStateRequest::new_id(SUCCESS_SWAP_MESSAGE_HASH.to_string());
        let traces = TraceResponse::mock_block_trace(0);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(!update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_status_failed_trace_action() {
        let request = TransactionStateRequest::new_id(FAILED_SWAP_MESSAGE_HASH.to_string());
        let traces = TraceResponse::mock_block_trace(1);
        let transaction = traces.root_transaction().unwrap().clone();

        let root_update = map_transaction_status(request.clone(), TraceResponse::mock(transaction, false, vec![])).unwrap();
        assert_eq!(root_update.state, TransactionState::Confirmed);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Reverted);
        assert!(!update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_status_incomplete_trace() {
        let request = TransactionStateRequest::new_id("hash".to_string());
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();
        let traces = TraceResponse::mock(
            transactions.transactions.first().unwrap().clone(),
            true,
            vec![TraceAction {
                success: Some(true),
                action_type: None,
                details: None,
            }],
        );

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Pending);
        assert!(!update.changes.is_empty());
    }
}
