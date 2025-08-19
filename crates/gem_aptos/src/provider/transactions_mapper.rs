use crate::{models::TransactionResponse, Transaction};
use primitives::{TransactionChange, TransactionState, TransactionUpdate};
use std::error::Error;

pub fn map_transaction_broadcast(response: &TransactionResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &response.message {
        return Err(message.clone().into());
    }

    response.hash.clone().ok_or_else(|| "Transaction response missing hash".into())
}

pub fn map_transaction_status(transaction: &Transaction) -> TransactionUpdate {
    let state = if transaction.success {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };

    let mut update = TransactionUpdate::new_state(state);

    if let (Some(gas_used), Some(gas_unit_price)) = (transaction.gas_used, transaction.gas_unit_price) {
        let fee = gas_used * gas_unit_price;
        update.changes.push(TransactionChange::NetworkFee(fee.to_string()));
    }

    update
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionResponse;

    #[test]
    fn test_map_transaction_broadcast() {
        let response = TransactionResponse {
            hash: Some("0xabc123".to_string()),
            message: None,
            error_code: None,
            vm_error_code: None,
        };

        let result = map_transaction_broadcast(&response).unwrap();
        assert_eq!(result, "0xabc123");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response = TransactionResponse {
            hash: None,
            message: Some("Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS".to_string()),
            error_code: Some("vm_error".to_string()),
            vm_error_code: Some(14),
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS"
        );
    }

    #[test]
    fn test_map_transaction_broadcast_from_testdata() {
        let json_data = include_str!("../../testdata/invalid_transaction_response.json");
        let response: TransactionResponse = serde_json::from_str(json_data).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS"
        );
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        use crate::Transaction;

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
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee("100".to_string())]);
        // 100 * 1 = 100
    }

    #[test]
    fn test_map_transaction_status_failed() {
        use crate::Transaction;

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
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee("50".to_string())]);
        // 50 * 1 = 50
    }
}
