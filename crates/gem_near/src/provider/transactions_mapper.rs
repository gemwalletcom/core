use primitives::{TransactionUpdate, TransactionState, TransactionChange};
use std::error::Error;
use crate::models::transaction::NearBroadcastResult;

pub fn map_transaction_broadcast(response: &NearBroadcastResult) -> Result<String, Box<dyn Error + Sync + Send>> {
    match response.final_execution_status.as_str() {
        "FINAL" => Ok(response.transaction.hash.clone()),
        _ => Err(format!("Broadcast failed with status: {}", response.final_execution_status).into())
    }
}

pub fn map_transaction_status(response: &NearBroadcastResult) -> TransactionUpdate {
    let state = match response.final_execution_status.as_str() {
        "FINAL" => TransactionState::Confirmed,
        _ => TransactionState::Failed,
    };
    
    let mut changes = vec![];
    if !response.transaction_outcome.outcome.tokens_burnt.is_empty() {
        changes.push(TransactionChange::NetworkFee(response.transaction_outcome.outcome.tokens_burnt.clone()));
    }
    
    TransactionUpdate {
        state,
        changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::JsonRpcResult;
    use crate::models::transaction::{NearBroadcastResult, NearBroadcastTransaction, NearTransactionOutcome, NearOutcome};

    fn create_test_transaction() -> NearBroadcastTransaction {
        NearBroadcastTransaction {
            hash: "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2".to_string(),
            signer_id: "test.near".to_string(),
            receiver_id: "receiver.near".to_string(),
            actions: vec![],
        }
    }

    fn create_test_outcome(tokens_burnt: &str) -> NearTransactionOutcome {
        NearTransactionOutcome {
            outcome: NearOutcome {
                tokens_burnt: tokens_burnt.to_string(),
            },
        }
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response = NearBroadcastResult {
            final_execution_status: "FINAL".to_string(),
            transaction: create_test_transaction(),
            transaction_outcome: create_test_outcome("417494768750000000000"),
        };
        
        let result = map_transaction_broadcast(&response).unwrap();
        assert_eq!(result, "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2");
    }

    #[test]
    fn test_map_transaction_broadcast_failure() {
        let response = NearBroadcastResult {
            final_execution_status: "EXECUTION_FAILURE".to_string(),
            transaction: create_test_transaction(),
            transaction_outcome: create_test_outcome("0"),
        };
        
        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("EXECUTION_FAILURE"));
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let response = NearBroadcastResult {
            final_execution_status: "FINAL".to_string(),
            transaction: create_test_transaction(),
            transaction_outcome: create_test_outcome("417494768750000000000"),
        };
        
        let result = map_transaction_status(&response);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
        if let TransactionChange::NetworkFee(fee) = &result.changes[0] {
            assert_eq!(fee, "417494768750000000000");
        }
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let response = NearBroadcastResult {
            final_execution_status: "EXECUTION_FAILURE".to_string(),
            transaction: create_test_transaction(),
            transaction_outcome: create_test_outcome("0"),
        };
        
        let result = map_transaction_status(&response);
        assert_eq!(result.state, TransactionState::Failed);
    }

    #[test]
    fn test_map_real_transaction_response() {
        let data = include_str!("../../testdata/successful_transaction.json");
        let response: JsonRpcResult<NearBroadcastResult> = serde_json::from_str(data).unwrap();
        
        let hash = map_transaction_broadcast(&response.result).unwrap();
        assert_eq!(hash, "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2");
        
        let status_update = map_transaction_status(&response.result);
        assert_eq!(status_update.state, TransactionState::Confirmed);
        assert_eq!(status_update.changes.len(), 1);
    }
}