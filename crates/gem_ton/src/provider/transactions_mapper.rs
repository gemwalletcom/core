use crate::constants::FAILED_OPERATION_OPCODES;
use crate::models::{BroadcastTransaction, HasMemo, MessageTransactions, TransactionMessage};
use chrono::DateTime;
use primitives::{chain::Chain, Transaction, TransactionChange, TransactionState, TransactionStateRequest, TransactionType, TransactionUpdate};
use std::error::Error;
use tonlib_core::TonAddress;

pub fn map_transaction_status(_request: TransactionStateRequest, transactions: MessageTransactions) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
    let transaction = transactions.transactions.first().ok_or("Transaction not found")?;
    let state = map_transaction_state(transaction);

    let fee = transaction.total_fees.clone();

    Ok(TransactionUpdate::new(state, vec![TransactionChange::NetworkFee(fee.into())]))
}

pub fn map_transaction_broadcast(broadcast_result: BroadcastTransaction) -> Result<String, Box<dyn Error + Sync + Send>> {
    let hash_bytes = base64::prelude::Engine::decode(&base64::prelude::BASE64_STANDARD, &broadcast_result.hash)?;
    Ok(hex::encode(hash_bytes))
}

fn map_transaction_state(transaction: &TransactionMessage) -> TransactionState {
    if let Some(description) = &transaction.description {
        if description.aborted {
            return TransactionState::Failed;
        }
        if let Some(compute_phase) = &description.compute_ph {
            // If success is None or false, or if exit_code indicates failure
            if !compute_phase.success.unwrap_or(false) {
                return TransactionState::Failed;
            }
            if let Some(exit_code) = compute_phase.exit_code {
                if exit_code != 0 && exit_code != 1 {
                    return TransactionState::Failed;
                }
            }
        }
        if let Some(action) = &description.action {
            if !action.success.unwrap_or(false) {
                return TransactionState::Failed;
            }
        }
    }

    if transaction.out_msgs.is_empty() {
        return TransactionState::Failed;
    }

    // TODO: Check for bounce/bounced fields when available in OutMessage struct
    // if transaction.out_msgs.iter().any(|msg| msg.bounce && msg.bounced) {
    //     return TransactionState::Failed;
    // }

    if let Some(in_msg) = &transaction.in_msg {
        if let Some(opcode) = &in_msg.opcode {
            if FAILED_OPERATION_OPCODES.contains(&opcode.as_str()) {
                return TransactionState::Failed;
            }
        }
    }

    TransactionState::Confirmed
}

pub fn map_transactions(transactions: Vec<TransactionMessage>) -> Vec<Transaction> {
    transactions.into_iter().filter_map(map_transaction_message).collect()
}

fn map_transaction_message(transaction: TransactionMessage) -> Option<Transaction> {
    let asset_id = Chain::Ton.as_asset_id();
    let state = map_transaction_state(&transaction);
    let created_at = DateTime::from_timestamp(0, 0)?; // TransactionMessage doesn't have utime field
    let hash = transaction.hash.clone();

    // Handle outgoing transfers (with out messages)
    if transaction.out_msgs.len() == 1 && is_simple_transfer(transaction.out_msgs.first()?) {
        let out_message = transaction.out_msgs.first()?;
        let from = parse_address(&out_message.source)?;
        let to = match &out_message.destination {
            Some(destination) => parse_address(destination)?,
            None => return None,
        };
        let value = out_message.value.as_ref().unwrap_or(&"0".to_string()).clone();
        let memo = extract_memo(out_message);

        return Some(Transaction::new(
            hash,
            asset_id.clone(),
            from,
            to,
            None,
            TransactionType::Transfer,
            state,
            transaction.total_fees.to_string(),
            asset_id,
            value,
            memo,
            None,
            created_at,
        ));
    }

    // Handle incoming transfers (with in message but no out messages)
    if transaction.out_msgs.is_empty() {
        if let Some(in_msg) = &transaction.in_msg {
            if let (Some(value), Some(source)) = (&in_msg.value, &in_msg.source) {
                if let Ok(value_int) = value.parse::<i64>() {
                    if value_int > 0 {
                        let from = parse_address(source)?;
                        let to = parse_address(&in_msg.destination)?;

                        return Some(Transaction::new(
                            hash,
                            asset_id.clone(),
                            from,
                            to,
                            None,
                            TransactionType::Transfer,
                            state,
                            transaction.total_fees.to_string(),
                            asset_id,
                            value.clone(),
                            None, // TransactionInMessage doesn't have memo fields
                            None,
                            created_at,
                        ));
                    }
                }
            }
        }
    }

    None
}

fn parse_address(address: &str) -> Option<String> {
    Some(TonAddress::from_hex_str(address).ok()?.to_base64_url())
}

fn is_simple_transfer(out_message: &crate::models::OutMessage) -> bool {
    match &out_message.op_code {
        None => true,
        Some(op_code) => op_code == "0x00000000" || op_code == "0x0",
    }
}

fn extract_memo<T: HasMemo>(message: &T) -> Option<String> {
    if let Some(comment) = message.comment() {
        if !comment.is_empty() {
            return Some(comment.clone());
        }
    }

    if let Some(decoded_body) = message.decoded_body() {
        if let Some(text) = &decoded_body.text {
            if !text.is_empty() {
                return Some(text.clone());
            }
        }
        if let Some(comment) = &decoded_body.comment {
            if !comment.is_empty() {
                return Some(comment.clone());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_transfer_state_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_transaction_status_response_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_status_response.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_jetton_transfer_failed() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_error.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "ZEC9rE/pUvEHGAJVzDn/6QdWevOOR4sA2dN4BaTA8hQ=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Failed);
    }

    #[test]
    fn test_jetton_transfer_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "X2rQTJQF38kXLWdQL42pP8NKrd2X1YDyp5h7Erq7sBA=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_jetton_transfer_success_2() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_success_2.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "pI2WtPJ6516pwuNti1h+Hetg0NZ8C/kBOboRkayUKL8=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_swap_ton_jetton_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_swap_ton_jetton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "wsQ2mvEWkMbw3QnyeBl85O+uuUsDNfuWJnc2mBh8lPg=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_swap_jetton_ton_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_swap_jetton_ton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "psAXHb1HyvR53f9LHmOzQWohJu3tDRWbxvZbHB1B+iY=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_transaction_with_null_values() {
        let transaction_json = include_str!("../../testdata/transaction_null_values.json");
        let transaction: TransactionMessage = serde_json::from_str(transaction_json).unwrap();

        assert_eq!(transaction.hash, "MhO9bk6+qCMfveyGBQYvoklath4SA7F/LegdwACJAvg=");
        assert_eq!(transaction.out_msgs.len(), 2);

        assert_eq!(transaction.out_msgs[0].value, None);
        assert_eq!(transaction.out_msgs[0].destination, None);

        assert_eq!(transaction.out_msgs[1].value, Some("137245095".to_string()));
    }

    #[test]
    fn test_map_get_transactions_by_block() {
        let block_transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/block_transactions.json")).unwrap();

        assert_eq!(block_transactions.transactions.len(), 18);

        let transactions = map_transactions(block_transactions.transactions);

        assert!(!transactions.is_empty());
        assert!(transactions.len() < 18);

        for transaction in &transactions {
            assert!(!transaction.hash.is_empty());
            assert!(!transaction.from.is_empty());
            assert!(!transaction.to.is_empty());
        }
    }
}
