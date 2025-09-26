use crate::STAKE_DEPOSIT_EVENT;
use crate::models::{Transaction, TransactionResponse};
use chrono::DateTime;
use num_bigint::BigUint;
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionState, TransactionType};
use std::error::Error;

pub fn map_transaction_broadcast(response: &TransactionResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &response.message {
        return Err(message.clone().into());
    }

    response.hash.clone().ok_or_else(|| "Transaction response missing hash".into())
}

pub fn map_transactions(transactions: Vec<Transaction>) -> Vec<PrimitivesTransaction> {
    let mut transactions = transactions.into_iter().flat_map(map_transaction).collect::<Vec<_>>();

    transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    transactions
}

pub fn map_transaction(transaction: Transaction) -> Option<PrimitivesTransaction> {
    let chain = Chain::Aptos;
    let events = transaction.clone().events.unwrap_or_default();

    if transaction.transaction_type.as_deref() == Some("user_transaction") && events.len() <= 4 {
        let deposit_event = events.iter().find(|x| x.event_type == STAKE_DEPOSIT_EVENT)?;

        let asset_id = chain.as_asset_id();
        let state = if transaction.success {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let to = &deposit_event.guid.account_address;
        let value = &deposit_event.get_amount()?;
        let gas_used = BigUint::from(transaction.gas_used.unwrap_or_default());
        let gas_unit_price = BigUint::from(transaction.gas_unit_price.unwrap_or_default());
        let fee = gas_used * gas_unit_price;
        let created_at = DateTime::from_timestamp_micros(transaction.timestamp as i64)?;

        let transaction = PrimitivesTransaction::new(
            transaction.hash.unwrap_or_default(),
            asset_id.clone(),
            transaction.sender.unwrap_or_default(),
            to.clone(),
            None,
            TransactionType::Transfer,
            state,
            fee.to_string(),
            asset_id,
            value.clone(),
            None,
            None,
            created_at,
        );
        return Some(transaction);
    }
    None
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
}
