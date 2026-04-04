use crate::models::TransactionStatus;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

pub fn map_transaction_status(transaction: &TransactionStatus) -> TransactionUpdate {
    let confirmed_round = transaction.confirmed_round.unwrap_or(0);
    let has_pool_error = transaction
        .pool_error
        .as_ref()
        .is_some_and(|error| !error.trim().is_empty());
    let state = if confirmed_round > 0 {
        TransactionState::Confirmed
    } else if has_pool_error {
        TransactionState::Failed
    } else {
        TransactionState::Pending
    };

    let mut changes = Vec::new();
    if confirmed_round > 0 {
        changes.push(TransactionChange::BlockNumber(confirmed_round.to_string()));
    }

    TransactionUpdate { state, changes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionStatus;
    use primitives::{TransactionChange, TransactionState};

    #[test]
    fn test_map_transaction_status_confirmed() {
        let result = map_transaction_status(&TransactionStatus {
            confirmed_round: Some(52961610),
            pool_error: None,
        });
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::BlockNumber("52961610".to_string())]);
    }

    #[test]
    fn test_map_transaction_status_success_data() {
        let status: TransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_transfer_success.json")).unwrap();
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::BlockNumber("52961610".to_string())]);
    }

    #[test]
    fn test_map_transaction_status_pending_data() {
        let status: TransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_transfer_pending.json")).unwrap();
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_map_transaction_status_failed_with_pool_error() {
        let result = map_transaction_status(&TransactionStatus {
            confirmed_round: None,
            pool_error: Some("overspend".to_string()),
        });
        assert_eq!(result.state, TransactionState::Failed);
        assert_eq!(result.changes.len(), 0);
    }
}
