use crate::rpc::model::{TransactionReceiptData, TronTransactionBroadcast};
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};
use std::error::Error;

pub fn map_transaction_broadcast(response: &TronTransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(txid) = &response.txid {
        Ok(txid.clone())
    } else if let (Some(code), Some(message)) = (&response.code, &response.message) {
        Err(format!("Broadcast failed [{}]: {}", code, message).into())
    } else {
        Err("Transaction broadcast failed with unknown error".into())
    }
}

pub fn map_transaction_status(receipt: &TransactionReceiptData) -> TransactionUpdate {
    if let Some(receipt_result) = &receipt.receipt.result {
        if receipt_result == "OUT_OF_ENERGY" || receipt_result == "FAILED" {
            return TransactionUpdate::new_state(TransactionState::Reverted);
        }
    }

    if receipt.block_number > 0 {
        let mut changes = vec![];
        if let Some(fee) = receipt.fee {
            changes.push(TransactionChange::NetworkFee(BigInt::from(fee)));
        }
        return TransactionUpdate::new(TransactionState::Confirmed, changes);
    }

    TransactionUpdate::new_state(TransactionState::Pending)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{TransactionReceipt, TransactionReceiptData, TronTransactionBroadcast};

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response = TronTransactionBroadcast {
            txid: Some("ABC123".to_string()),
            code: None,
            message: None,
        };

        assert_eq!(map_transaction_broadcast(&response).unwrap(), "ABC123");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response = TronTransactionBroadcast {
            txid: None,
            code: Some("INVALID_TX".to_string()),
            message: Some("Transaction validation failed".to_string()),
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Broadcast failed [INVALID_TX]: Transaction validation failed");
    }

    #[test]
    fn test_map_transaction_broadcast_unknown_error() {
        let response = TronTransactionBroadcast {
            txid: None,
            code: None,
            message: None,
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction broadcast failed with unknown error");
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1234567890,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(1000i64));
        }
    }

    #[test]
    fn test_map_transaction_status_reverted_out_of_energy() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(500),
            block_number: 12345,
            block_time_stamp: 1234567890,
            receipt: TransactionReceipt {
                result: Some("OUT_OF_ENERGY".to_string()),
            },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_map_transaction_status_reverted_failed() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(250),
            block_number: 12345,
            block_time_stamp: 1234567890,
            receipt: TransactionReceipt {
                result: Some("FAILED".to_string()),
            },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(0),
            block_number: 0,
            block_time_stamp: 0,
            receipt: TransactionReceipt { result: None },
            log: None,
        };

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes.len(), 0);
    }
}
