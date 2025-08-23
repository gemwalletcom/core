use crate::rpc::constants::{RECEIPT_FAILED, RECEIPT_OUT_OF_ENERGY};
use crate::rpc::model::{TransactionReceiptData, TronTransactionBroadcast};
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};
use std::error::Error;

fn decode_hex_message(hex_str: &str) -> String {
    match hex::decode(hex_str) {
        Ok(bytes) => String::from_utf8(bytes).unwrap_or_else(|_| hex_str.to_string()),
        Err(_) => hex_str.to_string(),
    }
}

pub fn map_transaction_broadcast(response: &TronTransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &response.message {
        Err(decode_hex_message(message).into())
    } else if let Some(txid) = &response.txid {
        Ok(txid.clone())
    } else {
        Err("Transaction broadcast failed with unknown error".into())
    }
}

pub fn map_transaction_status(receipt: &TransactionReceiptData) -> TransactionUpdate {
    if let Some(receipt_result) = &receipt.receipt.result {
        if receipt_result == RECEIPT_OUT_OF_ENERGY || receipt_result == RECEIPT_FAILED {
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
    fn test_map_transaction_broadcast_error() {
        let response: TronTransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_error.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Contract validate error : Cannot transfer TRX to yourself.");
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response: TronTransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_success.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "7f60ccd0594b5c3e0264cca9a6e6e64cb96ee66ce3a796b4356cb8ccc548f62b");
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
                result: Some(RECEIPT_OUT_OF_ENERGY.to_string()),
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
                result: Some(RECEIPT_FAILED.to_string()),
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
