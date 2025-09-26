use crate::models::Transaction;
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

pub fn map_transaction_status(transaction: &Transaction) -> TransactionUpdate {
    let state = if transaction.success {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };

    let mut update = TransactionUpdate::new_state(state);

    if let (Some(gas_used), Some(gas_unit_price)) = (transaction.gas_used, transaction.gas_unit_price) {
        let fee = gas_used * gas_unit_price;
        update.changes.push(TransactionChange::NetworkFee(BigInt::from(fee)));
    }

    update
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_status_confirmed() {
        let transaction = Transaction {
            hash: Some("0xabc123".to_string()),
            sender: Some("0x123".to_string()),
            success: true,
            gas_used: Some(100),
            gas_unit_price: Some(1),
            events: None,
            transaction_type: Some("user_transaction".to_string()),
            sequence_number: Some("1".to_string()),
            timestamp: 1234567890,
        };

        let result = map_transaction_status(&transaction);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(100u64))]);
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let transaction = Transaction {
            hash: Some("0xdef456".to_string()),
            sender: Some("0x456".to_string()),
            success: false,
            gas_used: Some(50),
            gas_unit_price: Some(1),
            events: None,
            transaction_type: Some("user_transaction".to_string()),
            sequence_number: Some("2".to_string()),
            timestamp: 1234567891,
        };

        let result = map_transaction_status(&transaction);
        assert_eq!(result.state, TransactionState::Failed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(50u64))]);
    }
}
