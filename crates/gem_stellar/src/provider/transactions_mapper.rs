use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};
use crate::models::transaction::{Payment, StellarTransactionBroadcast, StellarTransactionStatus};
use chrono::DateTime;
use num_bigint::BigInt;
use primitives::{Transaction, TransactionChange, TransactionState, TransactionType, TransactionUpdate, chain::Chain};
use std::error::Error;
use url::form_urlencoded;

pub fn encode_transaction_data(data: &str) -> String {
    form_urlencoded::Serializer::new(String::new()).append_pair("tx", data).finish()
}

pub fn map_transaction_broadcast(response: &StellarTransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(hash) = &response.hash {
        Ok(hash.clone())
    } else if let Some(error) = &response.error_message {
        Err(format!("Broadcast error: {}", error).into())
    } else {
        Err("Unknown broadcast error".into())
    }
}

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

pub fn map_transactions(chain: Chain, transactions: Vec<Payment>) -> Vec<Transaction> {
    transactions.into_iter().flat_map(|x| map_transaction(chain, x)).collect()
}

pub fn map_transaction(chain: Chain, transaction: Payment) -> Option<Transaction> {
    match transaction.payment_type.as_str() {
        TRANSACTION_TYPE_PAYMENT | TRANSACTION_TYPE_CREATE_ACCOUNT => {
            if transaction.clone().asset_type.unwrap_or_default() == "native" || transaction.clone().payment_type.as_str() == TRANSACTION_TYPE_CREATE_ACCOUNT {
                let created_at = DateTime::parse_from_rfc3339(&transaction.created_at).ok()?.into();

                return Some(Transaction::new(
                    transaction.clone().transaction_hash,
                    chain.as_asset_id(),
                    transaction.from_address()?,
                    transaction.to_address()?,
                    None,
                    TransactionType::Transfer,
                    transaction.get_state(),
                    "1000".to_string(), // TODO: Calculate from block/transaction
                    chain.as_asset_id(),
                    transaction.get_value()?,
                    transaction.clone().get_memo(),
                    None,
                    created_at,
                ));
            }

            None
        }
        _ => None,
    }
}

pub fn is_token_address(token_id: &str) -> bool {
    token_id.len() > 32 && token_id.contains('-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{StellarTransactionBroadcast, StellarTransactionStatus};
    use num_bigint::{BigInt, BigUint};
    use primitives::TransactionChange;

    #[test]
    fn test_encode_transaction_data_variants() {
        assert_eq!(encode_transaction_data("payload"), "tx=payload");

        let special = encode_transaction_data("tx=abc/123?&value=42 +more");
        assert_eq!(special, "tx=tx%3Dabc%2F123%3F%26value%3D42+%2Bmore");
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response = StellarTransactionBroadcast {
            hash: Some("test_hash_123".to_string()),
            error_message: None,
        };

        let result = map_transaction_broadcast(&response).unwrap();
        assert_eq!(result, "test_hash_123");
    }

    #[test]
    fn test_map_transaction_status() {
        let status = StellarTransactionStatus {
            successful: true,
            fee_charged: BigUint::from(1000u64),
            hash: "test_hash_123".to_string(),
        };

        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
    }

    #[test]
    fn test_map_transaction_broadcast_with_real_data() {
        let data = include_str!("../../testdata/transaction_transfer_broadcast_success.json");
        let response: StellarTransactionStatus = serde_json::from_str(data).unwrap();

        let result = map_transaction_broadcast(&StellarTransactionBroadcast {
            hash: Some(response.hash.clone()),
            error_message: None,
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dbc69dff72e4ca7ddf47311e12da09ac5952c777d19855f95f13b0ec624f8baf");
    }

    #[test]
    fn test_map_transaction_status_with_real_data() {
        let data = include_str!("../../testdata/transaction_state_success.json");
        let response: StellarTransactionStatus = serde_json::from_str(data).unwrap();

        let result = map_transaction_status(&response);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);

        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, &BigInt::from(100u64));
        } else {
            panic!("Expected NetworkFee change");
        }
    }
}
