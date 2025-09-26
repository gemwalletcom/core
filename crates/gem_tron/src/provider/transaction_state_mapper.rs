use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

use crate::models::TransactionReceiptData;
use crate::rpc::constants::{RECEIPT_FAILED, RECEIPT_OUT_OF_ENERGY};

pub fn map_transaction_status(receipt: &TransactionReceiptData) -> TransactionUpdate {
    if let Some(receipt_result) = &receipt.receipt.result
        && (receipt_result == RECEIPT_OUT_OF_ENERGY || receipt_result == RECEIPT_FAILED)
    {
        return TransactionUpdate::new_state(TransactionState::Reverted);
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
    use crate::models::{TransactionReceipt, TransactionReceiptData};

    fn create_receipt(result: Option<&str>, block_number: i64, fee: Option<i64>) -> TransactionReceiptData {
        TransactionReceiptData {
            id: "tx_id".to_string(),
            fee,
            block_number,
            block_time_stamp: 0,
            receipt: TransactionReceipt {
                result: result.map(|value| value.to_string()),
            },
            log: None,
        }
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let receipt = create_receipt(None, 10, Some(100));

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert!(!result.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_status_reverted_out_of_energy() {
        let receipt = create_receipt(Some(RECEIPT_OUT_OF_ENERGY), 10, Some(100));

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Reverted);
    }

    #[test]
    fn test_map_transaction_status_reverted_failed() {
        let receipt = create_receipt(Some(RECEIPT_FAILED), 10, Some(100));

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Reverted);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let receipt = create_receipt(None, 0, None);

        let result = map_transaction_status(&receipt);
        assert_eq!(result.state, TransactionState::Pending);
    }
}
