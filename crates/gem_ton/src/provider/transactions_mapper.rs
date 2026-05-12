use crate::address::Address;
use crate::address::hex_to_base64_address;
use crate::constants::FAILED_OPERATION_OPCODES;
use crate::models::{BroadcastTransaction, JettonSwapDetails, OutMessage, TRACE_ACTION_JETTON_SWAP, Trace, TraceAction, TransactionMessage};
use chrono::DateTime;
use gem_encoding::decode_base64;
use primitives::{AssetId, Transaction, TransactionState, TransactionSwapMetadata, TransactionType, chain::Chain};
use std::error::Error;

pub fn map_transaction_broadcast(broadcast_result: BroadcastTransaction) -> Result<String, Box<dyn Error + Sync + Send>> {
    let hash_bytes = decode_base64(&broadcast_result.hash)?;
    Ok(hex::encode(hash_bytes))
}

pub(crate) fn map_transaction_state(transaction: &TransactionMessage) -> TransactionState {
    if let Some(description) = &transaction.description {
        if description.aborted {
            return TransactionState::Failed;
        }
        if let Some(compute_phase) = &description.compute_ph {
            if !compute_phase.success.unwrap_or(false) {
                return TransactionState::Failed;
            }
            if let Some(exit_code) = compute_phase.exit_code
                && exit_code != 0
                && exit_code != 1
            {
                return TransactionState::Failed;
            }
        }
        if let Some(action) = &description.action
            && !action.success.unwrap_or(false)
        {
            return TransactionState::Failed;
        }
    }

    if transaction.out_msgs.is_empty() {
        return TransactionState::Failed;
    }

    if let Some(in_msg) = &transaction.in_msg
        && let Some(opcode) = &in_msg.opcode
        && FAILED_OPERATION_OPCODES.contains(&opcode.as_str())
    {
        return TransactionState::Reverted;
    }

    TransactionState::Confirmed
}

pub(crate) fn base64_hash_to_hex(base64_hash: &str) -> Option<String> {
    decode_base64(base64_hash).ok().map(hex::encode)
}

pub fn map_trace_transactions(traces: Vec<Trace>) -> Vec<Transaction> {
    traces.into_iter().filter_map(map_root_trace_transaction).collect()
}

fn map_root_trace_transaction(trace: Trace) -> Option<Transaction> {
    let state = if trace.is_incomplete || trace.has_actions() {
        Some(trace.action_state())
    } else {
        None
    };
    let swap = jetton_swap(&trace.actions);
    let mut transactions = trace.transactions;
    let root_hash = trace.transactions_order.into_iter().next()?;
    let root = transactions.remove(&root_hash)?;
    let mut transaction = map_transaction_message_with_state(root, state)?;
    if let Some((sender, metadata)) = swap
        && let Ok(value) = serde_json::to_value(metadata)
    {
        transaction.transaction_type = TransactionType::Swap;
        transaction.from = sender.clone();
        transaction.to = sender;
        transaction.metadata = Some(value);
    }
    Some(transaction)
}

fn jetton_swap(actions: &[TraceAction]) -> Option<(String, TransactionSwapMetadata)> {
    let action = actions
        .iter()
        .find(|action| action.action_type.as_deref() == Some(TRACE_ACTION_JETTON_SWAP) && action.success == Some(true))?;
    let details: JettonSwapDetails = serde_json::from_value(action.details.clone()?).ok()?;
    let sender = parse_address(&details.sender)?;
    let metadata = TransactionSwapMetadata {
        from_asset: ton_asset_id(details.asset_in.as_deref())?,
        from_value: details.dex_incoming_transfer.amount,
        to_asset: ton_asset_id(details.asset_out.as_deref())?,
        to_value: details.dex_outgoing_transfer.amount,
        provider: details.dex,
    };
    Some((sender, metadata))
}

fn ton_asset_id(raw_address: Option<&str>) -> Option<AssetId> {
    match raw_address {
        None => Some(AssetId::from_chain(Chain::Ton)),
        Some(hex_address) => hex_to_base64_address(hex_address).map(|token_id| AssetId::from_token(Chain::Ton, &token_id)),
    }
}

fn map_transaction_message_with_state(transaction: TransactionMessage, state: Option<TransactionState>) -> Option<Transaction> {
    let asset_id = Chain::Ton.as_asset_id();
    let state = state.unwrap_or_else(|| map_transaction_state(&transaction));
    let created_at = DateTime::from_timestamp(transaction.now, 0)?;
    let hash = base64_hash_to_hex(&transaction.hash)?;

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

    if transaction.out_msgs.is_empty()
        && let Some(in_msg) = &transaction.in_msg
        && let (Some(value), Some(source)) = (&in_msg.value, &in_msg.source)
        && let Ok(value_int) = value.parse::<i64>()
        && value_int > 0
    {
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
            None,
            None,
            created_at,
        ));
    }

    None
}

fn parse_address(address: &str) -> Option<String> {
    Address::try_parse_hex(address).map(|a| a.encode_non_bounceable())
}

fn is_simple_transfer(out_message: &crate::models::OutMessage) -> bool {
    match &out_message.op_code {
        None => true,
        Some(op_code) => op_code == "0x00000000" || op_code == "0x0",
    }
}

fn extract_memo(message: &OutMessage) -> Option<String> {
    if let Some(comment) = &message.comment
        && !comment.is_empty()
    {
        return Some(comment.clone());
    }

    if let Some(decoded_body) = &message.decoded_body {
        if let Some(text) = &decoded_body.text
            && !text.is_empty()
        {
            return Some(text.clone());
        }
        if let Some(comment) = &decoded_body.comment
            && !comment.is_empty()
        {
            return Some(comment.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MessageTransactions, TraceResponse};
    use crate::provider::testkit::{FAILED_SWAP_ROOT_TRANSACTION_HEX_HASH, SUCCESS_SWAP_ROOT_TRANSACTION_HEX_HASH, TEST_TRANSACTION_ID};

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
        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);

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
        assert_eq!(state, TransactionState::Reverted);
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
        let transaction: TransactionMessage = serde_json::from_str(include_str!("../../testdata/transaction_null_values.json")).unwrap();

        assert_eq!(transaction.hash, "MhO9bk6+qCMfveyGBQYvoklath4SA7F/LegdwACJAvg=");
        assert_eq!(transaction.out_msgs.len(), 2);

        assert_eq!(transaction.out_msgs[0].value, None);
        assert_eq!(transaction.out_msgs[0].destination, None);

        assert_eq!(transaction.out_msgs[1].value, Some("137245095".to_string()));
    }

    #[test]
    fn test_map_trace_transactions_jetton_swap() {
        let traces = TraceResponse::mock_jetton_swap();
        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.from, "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV");
        assert_eq!(transaction.from, transaction.to);

        let metadata = transaction.metadata.as_ref().expect("swap metadata");
        let swap: TransactionSwapMetadata = serde_json::from_value(metadata.clone()).unwrap();
        assert_eq!(swap.from_asset, AssetId::from_chain(Chain::Ton));
        assert_eq!(swap.from_value, "1000000000");
        assert_eq!(swap.to_asset.chain, Chain::Ton);
        assert!(swap.to_asset.token_id.is_some());
        assert_eq!(swap.to_value, "2436222");
        assert_eq!(swap.provider.as_deref(), Some("stonfi_v2"));
    }

    #[test]
    fn test_map_trace_transactions_by_block() {
        let traces = TraceResponse::mock_block_traces();

        assert_eq!(traces.traces.len(), 2);

        let transactions = map_trace_transactions(traces.traces);
        let hashes = transactions.iter().map(|transaction| transaction.hash.as_str()).collect::<Vec<_>>();

        assert_eq!(hashes, vec![SUCCESS_SWAP_ROOT_TRANSACTION_HEX_HASH, FAILED_SWAP_ROOT_TRANSACTION_HEX_HASH]);
        assert_eq!(transactions[0].state, TransactionState::Confirmed);
        assert_eq!(transactions[1].state, TransactionState::Reverted);
    }
}
