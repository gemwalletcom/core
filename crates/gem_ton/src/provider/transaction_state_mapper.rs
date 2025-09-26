use std::error::Error;

use primitives::{TransactionChange, TransactionStateRequest, TransactionUpdate};

use crate::models::MessageTransactions;
use crate::provider::transactions_mapper::map_transaction_state;

pub fn map_transaction_status(
    _request: TransactionStateRequest,
    transactions: MessageTransactions,
) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
    let transaction = transactions.transactions.first().ok_or("Transaction not found")?;
    let state = map_transaction_state(transaction);

    let fee = transaction.total_fees.clone();

    Ok(TransactionUpdate::new(state, vec![TransactionChange::NetworkFee(fee.into())]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::TransactionState;

    #[test]
    fn test_map_transaction_status_confirmed() {
        let request = TransactionStateRequest::new_id("hash".to_string());
        let transactions: MessageTransactions =
            serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();

        let update = map_transaction_status(request, transactions).unwrap();
        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(!update.changes.is_empty());
    }
}
