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
    use crate::models::transaction::{StellarTransactionLinks, StellarTransactionStatus};

    #[test]
    fn test_map_transaction_status() {
        let status = StellarTransactionStatus {
            hash: Some("hash".to_string()),
            ledger: Some(123),
            result_xdr: None,
            paging_token: Some("token".to_string()),
            successful: true,
            fee_charged: Some(100.into()),
            max_fee: None,
            memo_type: None,
            memo: None,
            envelope_xdr: None,
            result_meta_xdr: None,
            fee_meta_xdr: None,
            created_at: None,
            source_account: None,
            source_account_sequence: None,
            signatures: vec![],
            memo_bytes: None,
            links: StellarTransactionLinks::default(),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 2);
    }

    #[test]
    fn test_map_transaction_status_with_real_data() {
        let response: StellarTransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_status.json")).unwrap();

        let result = map_transaction_status(&response);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert!(!result.changes.is_empty());
    }
}
