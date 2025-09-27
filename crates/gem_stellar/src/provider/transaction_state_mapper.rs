use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

use crate::models::transaction::StellarTransactionStatus;

pub fn map_transaction_status(tx: &StellarTransactionStatus) -> TransactionUpdate {
    let state = if tx.successful {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };

    let network_fee = BigInt::from(tx.fee_charged.clone());

    TransactionUpdate {
        state,
        changes: vec![TransactionChange::NetworkFee(network_fee)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    use crate::models::transaction::StellarTransactionStatus;

    #[test]
    fn test_map_transaction_status() {
        let status = StellarTransactionStatus {
            successful: true,
            fee_charged: BigUint::from(100_u32),
            hash: "hash".to_string(),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
    }

    #[test]
    fn test_map_transaction_status_with_real_data() {
        let response: StellarTransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_state_success.json")).unwrap();

        let result = map_transaction_status(&response);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert!(!result.changes.is_empty());
    }
}
