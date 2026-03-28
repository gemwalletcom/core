use serde_json::json;

use crate::models::TransactionPayload;
use crate::{
    APTOS_TRANSFER_COINS_FUNCTION, APTOS_TRANSFER_FUNCTION, DELEGATION_POOL_ADD_STAKE_FUNCTION, DELEGATION_POOL_UNLOCK_FUNCTION, DELEGATION_POOL_WITHDRAW_FUNCTION,
    ENTRY_FUNCTION_PAYLOAD_TYPE,
};

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

pub fn build_transfer_coins_transaction_payload(token_id: &str, recipient: &str, amount: &str) -> TransactionPayload {
    TransactionPayload {
        function: Some(APTOS_TRANSFER_COINS_FUNCTION.to_string()),
        type_arguments: vec![token_id.to_string()],
        arguments: vec![json!(recipient), json!(amount)],
        payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
    }
}

pub fn build_fungible_transfer_transaction_payload(token_id: &str, recipient: &str, amount: &str) -> TransactionPayload {
    TransactionPayload {
        function: Some("0x1::primary_fungible_store::transfer".to_string()),
        type_arguments: vec!["0x1::object::ObjectCore".to_string()],
        arguments: vec![json!(token_id), json!(recipient), json!(amount)],
        payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
    }
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
}
