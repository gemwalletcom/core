use crate::models::Transaction;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};
use std::error::Error;

pub fn map_transaction_state(transaction: &Transaction) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
    let state = if transaction.success {
        TransactionState::Confirmed
    } else {
        TransactionState::Reverted
    };

    let mut changes = Vec::new();

    if let (Some(gas_used), Some(gas_unit_price)) = (transaction.gas_used, transaction.gas_unit_price) {
        let fee = gas_used * gas_unit_price;
        changes.push(TransactionChange::NetworkFee(fee.to_string()));
    }

    Ok(TransactionUpdate { state, changes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Transaction;

    #[test]
    fn test_map_transaction_state_success() {
        let transaction = Transaction {
            hash: Some("0x123".to_string()),
            sender: Some("0xabc".to_string()),
            success: true,
            gas_used: Some(1000),
            gas_unit_price: Some(100),
            events: None,
            transaction_type: Some("user_transaction".to_string()),
            sequence_number: Some("1".to_string()),
            timestamp: 1234567890,
        };

        let result = map_transaction_state(&transaction).unwrap();

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, "100000");
        } else {
            panic!("Expected NetworkFee change");
        }
    }

    #[test]
    fn test_map_transaction_state_reverted() {
        let transaction = Transaction {
            hash: Some("0x123".to_string()),
            sender: Some("0xabc".to_string()),
            success: false,
            gas_used: Some(500),
            gas_unit_price: Some(50),
            events: None,
            transaction_type: Some("user_transaction".to_string()),
            sequence_number: Some("1".to_string()),
            timestamp: 1234567890,
        };

        let result = map_transaction_state(&transaction).unwrap();

        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes.len(), 1);
    }
}
