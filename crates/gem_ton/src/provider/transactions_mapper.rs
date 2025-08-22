use crate::constants::FAILED_OPERATION_OPCODES;
use crate::rpc::model::{TonBroadcastTransaction, TonMessageTransactions, TonTransactionMessage};
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

pub fn map_transaction_status(
    _request: TransactionStateRequest,
    transactions: TonMessageTransactions,
) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
    let transaction = transactions.transactions.first().ok_or("Transaction not found")?;

    let state = map_transaction_state(transaction);

    let fee = transaction
        .total_fees
        .parse::<BigInt>()
        .map_err(|e| format!("Failed to parse total_fees: {}", e))?;

    Ok(TransactionUpdate::new(state, vec![TransactionChange::NetworkFee(fee)]))
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

    if transaction.out_msgs.iter().any(|msg| msg.bounce && msg.bounced) {
        return TransactionState::Failed;
    }

    if let Some(in_msg) = &transaction.in_msg {
        if let Some(opcode) = &in_msg.opcode {
            if FAILED_OPERATION_OPCODES.contains(&opcode.as_str()) {
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

    #[test]
    fn test_jetton_transfer_success() {
        let transactions: TonMessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "X2rQTJQF38kXLWdQL42pP8NKrd2X1YDyp5h7Erq7sBA=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_jetton_transfer_success_2() {
        let transactions: TonMessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_success_2.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "pI2WtPJ6516pwuNti1h+Hetg0NZ8C/kBOboRkayUKL8=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_swap_ton_jetton_success() {
        let transactions: TonMessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_swap_ton_jetton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "wsQ2mvEWkMbw3QnyeBl85O+uuUsDNfuWJnc2mBh8lPg=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_swap_jetton_ton_success() {
        let transactions: TonMessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_swap_jetton_ton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "psAXHb1HyvR53f9LHmOzQWohJu3tDRWbxvZbHB1B+iY=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }
}
