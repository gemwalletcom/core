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
    use num_bigint::BigUint;

    #[test]
    fn test_map_transaction_status_success() {
        let status = TransactionStatus {
            status: "success".to_string(),
            fee: BigUint::from(12_u32),
        };
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let status = TransactionStatus {
            status: "failed".to_string(),
            fee: BigUint::from(8_u32),
        };
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Failed);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let status = TransactionStatus {
            status: "unknown".to_string(),
            fee: BigUint::from(4_u32),
        };
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Pending);
    }
}
