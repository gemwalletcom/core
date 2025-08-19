use crate::rpc::model::{TonBroadcastTransaction, TonMessageTransactions, TonTransactionMessage};
use primitives::{TransactionChange, TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

pub fn map_transaction_status(
    _request: TransactionStateRequest,
    transactions: TonMessageTransactions,
) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
    let transaction = transactions
        .transactions
        .first()
        .ok_or("Transaction not found")?;

    let state = map_transaction_state(transaction);

    Ok(TransactionUpdate::new(state, vec![ TransactionChange::NetworkFee(transaction.total_fees.clone()) ]))
}

pub fn map_transaction_broadcast(broadcast_result: TonBroadcastTransaction) -> Result<String, Box<dyn Error + Sync + Send>> {
    let hash_bytes = base64::prelude::Engine::decode(&base64::prelude::BASE64_STANDARD, &broadcast_result.hash)?;
    Ok(hex::encode(hash_bytes))
}

fn map_transaction_state(transaction: &TonTransactionMessage) -> TransactionState {
    if let Some(description) = &transaction.description {
        if description.aborted {
            return TransactionState::Failed;
        }
        if let Some(compute_phase) = &description.compute_ph {
            if !compute_phase.success || (compute_phase.exit_code != 0 && compute_phase.exit_code != 1) {
                return TransactionState::Failed;
            }
        }
        if let Some(action) = &description.action {
            if !action.success {
                return TransactionState::Failed;
            }
        }
    }

    if transaction.out_msgs.is_empty() {
        return TransactionState::Failed;
    }

    // Check for bounced messages which indicate failure
    if transaction.out_msgs.iter().any(|msg| msg.bounce && msg.bounced) {
        return TransactionState::Failed;
    }
    
    for out_msg in &transaction.out_msgs {
        if let (Some(_opcode), Ok(value)) = (&out_msg.opcode, out_msg.value.parse::<u64>()) {
            if value >= 100_000_000 {  // >= 0.1 TON indicates likely refund
                return TransactionState::Failed;
            }
        }
    }

    TransactionState::Confirmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_transfer_state_success() {
        let transactions: TonMessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();
        
        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_transaction_status_response_success() {
        let transactions: TonMessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_status_response.json")).unwrap();
        
        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY=");
        assert_eq!(transaction.account, "0:33A14A5A9406979D59B9328898591660B8B1736342B11632EFDCC911AB9057CF");
        
        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_jetton_transfer_failed() {
        let transactions: TonMessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_error.json")).unwrap();
        
        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "ZEC9rE/pUvEHGAJVzDn/6QdWevOOR4sA2dN4BaTA8hQ=");
        
        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Failed);
    }
}
