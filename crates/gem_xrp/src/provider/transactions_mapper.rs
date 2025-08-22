use crate::rpc::model::{TransactionBroadcast, TransactionStatus};
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};
use std::error::Error;

pub fn map_transaction_broadcast(broadcast_result: &TransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(accepted) = broadcast_result.accepted
        && !accepted
    {
        if let Some(error_msg) = &broadcast_result.engine_result_message {
            return Err(format!("Transaction rejected: {}", error_msg).into());
        }
        return Err("Transaction was not accepted".into());
    }

    if let Some(hash) = &broadcast_result.hash {
        Ok(hash.clone())
    } else if let Some(tx_json) = &broadcast_result.tx_json {
        Ok(tx_json.hash.clone())
    } else {
        Err("Transaction broadcast failed - no hash returned".into())
    }
}

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
    use crate::rpc::model::{LedgerResult, TransactionBroadcast, TransactionStatus};
    use num_bigint::BigUint;

    #[test]
    fn test_map_transaction_broadcast_success() {
        let json_data = include_str!("../testdata/transaction_broadcast_success.json");
        let response: LedgerResult<TransactionBroadcast> = serde_json::from_str(json_data).expect("Failed to parse JSON");

        let result = map_transaction_broadcast(&response.result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "04F53F220DD1BCB7CCF279D66FFB986EA41383EFC9378CA1EBF1823D7C89264F");
    }

    #[test]
    fn test_map_transaction_broadcast_failed() {
        let json_data = include_str!("../testdata/transaction_broadcast_failed.json");
        let response: LedgerResult<TransactionBroadcast> = serde_json::from_str(json_data).expect("Failed to parse JSON");

        let result = map_transaction_broadcast(&response.result);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction rejected: Ledger sequence too high.");
    }

    #[test]
    fn test_map_transaction_status_success() {
        let status = TransactionStatus {
            status: "success".to_string(),
            fee: BigUint::from(100u64),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(100u64));
        }
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let status = TransactionStatus {
            status: "failed".to_string(),
            fee: BigUint::from(50u64),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Failed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(50u64));
        }
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let status = TransactionStatus {
            status: "pending".to_string(),
            fee: BigUint::from(75u64),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(75u64));
        }
    }
}
