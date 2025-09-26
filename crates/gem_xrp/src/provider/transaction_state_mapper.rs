use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

use crate::models::rpc::TransactionStatus;

pub fn map_transaction_status(status: &TransactionStatus) -> TransactionUpdate {
    let state = match status.status.as_str() {
        "success" => TransactionState::Confirmed,
        "failed" => TransactionState::Failed,
        _ => TransactionState::Pending,
    };

    let changes = vec![TransactionChange::NetworkFee(BigInt::from(status.fee.clone()))];

    TransactionUpdate { state, changes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_status_success() {
        let status: TransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_status_success.json")).unwrap();
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let status: TransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_status_failed.json")).unwrap();
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Failed);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let status: TransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_status_pending.json")).unwrap();
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Pending);
    }
}
