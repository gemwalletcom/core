use primitives::{TransactionUpdate, TransactionState, TransactionChange};
use std::error::Error;
use crate::typeshare::transaction::{StellarTransactionBroadcast, StellarTransactionStatus};

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
    
    let network_fee = tx.fee_charged.parse::<u64>().unwrap_or(0);
    
    TransactionUpdate {
        state,
        changes: vec![TransactionChange::NetworkFee(network_fee.to_string())],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typeshare::transaction::{StellarTransactionBroadcast, StellarTransactionStatus};

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
            fee_charged: "1000".to_string(),
        };
        
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes.len(), 1);
    }
}