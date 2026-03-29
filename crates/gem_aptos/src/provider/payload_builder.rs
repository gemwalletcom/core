use serde_json::json;

use crate::models::TransactionPayload;
use crate::token_id::is_fungible_asset_token_id;
use crate::{APTOS_TRANSFER_FUNCTION, DELEGATION_POOL_ADD_STAKE_FUNCTION, DELEGATION_POOL_UNLOCK_FUNCTION, DELEGATION_POOL_WITHDRAW_FUNCTION, ENTRY_FUNCTION_PAYLOAD_TYPE};

fn build_payload(function: &str, first_argument: &str, amount: &str) -> TransactionPayload {
    TransactionPayload {
        function: Some(function.to_string()),
        type_arguments: vec![],
        arguments: vec![json!(first_argument), json!(amount)],
        payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
    }
}

pub fn build_stake_transaction_payload(pool_address: &str, amount: &str) -> TransactionPayload {
    build_payload(DELEGATION_POOL_ADD_STAKE_FUNCTION, pool_address, amount)
}

pub fn build_unstake_transaction_payload(pool_address: &str, amount: &str) -> TransactionPayload {
    build_payload(DELEGATION_POOL_UNLOCK_FUNCTION, pool_address, amount)
}

pub fn build_withdraw_transaction_payload(pool_address: &str, amount: &str) -> TransactionPayload {
    build_payload(DELEGATION_POOL_WITHDRAW_FUNCTION, pool_address, amount)
}

pub fn build_transfer_transaction_payload(recipient: &str, amount: &str) -> TransactionPayload {
    build_payload(APTOS_TRANSFER_FUNCTION, recipient, amount)
}

pub fn build_fungible_transfer_transaction_payload(token_id: &str, recipient: &str, amount: &str) -> TransactionPayload {
    TransactionPayload {
        function: Some("0x1::primary_fungible_store::transfer".to_string()),
        type_arguments: vec!["0x1::object::ObjectCore".to_string()],
        arguments: vec![json!(token_id), json!(recipient), json!(amount)],
        payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
    }
}

pub fn build_token_transfer_transaction_payload(token_id: &str, recipient: &str, amount: &str) -> Result<TransactionPayload, &'static str> {
    if !is_fungible_asset_token_id(token_id) {
        return Err("Invalid Aptos token ID format");
    }

    Ok(build_fungible_transfer_transaction_payload(token_id, recipient, amount))
}

pub fn build_stake_payload_data(pool_address: &str, amount: &str) -> String {
    serde_json::to_string(&build_stake_transaction_payload(pool_address, amount)).unwrap()
}

pub fn build_unstake_payload_data(pool_address: &str, amount: &str) -> String {
    serde_json::to_string(&build_unstake_transaction_payload(pool_address, amount)).unwrap()
}

pub fn build_withdraw_payload_data(pool_address: &str, amount: &str) -> String {
    serde_json::to_string(&build_withdraw_transaction_payload(pool_address, amount)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const TEST_POOL_ADDRESS: &str = "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e";

    #[test]
    fn test_build_stake_transaction_payload() {
        let payload = build_stake_transaction_payload(TEST_POOL_ADDRESS, "100000000");
        let result: Value = serde_json::to_value(&payload).unwrap();
        let expected = serde_json::json!({
            "function": "0x1::delegation_pool::add_stake",
            "type_arguments": [],
            "arguments": [TEST_POOL_ADDRESS, "100000000"],
            "type": "entry_function_payload"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_unstake_transaction_payload() {
        let payload = build_unstake_transaction_payload(TEST_POOL_ADDRESS, "50000000");
        let result: Value = serde_json::to_value(&payload).unwrap();
        let expected = serde_json::json!({
            "function": "0x1::delegation_pool::unlock",
            "type_arguments": [],
            "arguments": [TEST_POOL_ADDRESS, "50000000"],
            "type": "entry_function_payload"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_withdraw_transaction_payload() {
        let payload = build_withdraw_transaction_payload(TEST_POOL_ADDRESS, "1102185008");
        let result: Value = serde_json::to_value(&payload).unwrap();
        let expected = serde_json::json!({
            "function": "0x1::delegation_pool::withdraw",
            "type_arguments": [],
            "arguments": [TEST_POOL_ADDRESS, "1102185008"],
            "type": "entry_function_payload"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_token_transfer_transaction_payload_fungible_asset() {
        let payload = build_token_transfer_transaction_payload("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b", TEST_POOL_ADDRESS, "1").unwrap();
        let result: Value = serde_json::to_value(&payload).unwrap();
        let expected = serde_json::json!({
            "function": "0x1::primary_fungible_store::transfer",
            "type_arguments": ["0x1::object::ObjectCore"],
            "arguments": ["0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b", TEST_POOL_ADDRESS, "1"],
            "type": "entry_function_payload"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_token_transfer_transaction_payload_invalid() {
        let err = build_token_transfer_transaction_payload("invalid", TEST_POOL_ADDRESS, "1").unwrap_err();
        assert_eq!(err, "Invalid Aptos token ID format");
    }
}
