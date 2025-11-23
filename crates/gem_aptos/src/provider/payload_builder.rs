use serde_json::json;

pub fn build_stake_payload(_delegator_address: &str, pool_address: &str, amount: &str) -> String {
    let payload = json!({
        "function": "0x1::delegation_pool::add_stake",
        "type_arguments": [],
        "arguments": [pool_address, amount],
        "type": "entry_function_payload"
    });
    payload.to_string()
}

pub fn build_unstake_payload(_delegator_address: &str, pool_address: &str, amount: &str) -> String {
    let payload = json!({
        "function": "0x1::delegation_pool::unlock",
        "type_arguments": [],
        "arguments": [pool_address, amount],
        "type": "entry_function_payload"
    });
    payload.to_string()
}

pub fn build_withdraw_payload(_delegator_address: &str, pool_address: &str, value: &str) -> String {
    let payload = json!({
        "function": "0x1::delegation_pool::withdraw",
        "type_arguments": [],
        "arguments": [pool_address, value],
        "type": "entry_function_payload"
    });
    payload.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const TEST_POOL_ADDRESS: &str = "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e";
    const TEST_DELEGATOR_ADDRESS: &str = "0x17a4c5bbe7b7e7d1359e031c01061413100a7f0352411b2a4d1be143301d685d";

    #[test]
    fn test_build_stake_payload() {
        let result = build_stake_payload(TEST_DELEGATOR_ADDRESS, TEST_POOL_ADDRESS, "100000000");
        let expected = json!({
            "function": "0x1::delegation_pool::add_stake",
            "type_arguments": [],
            "arguments": [TEST_POOL_ADDRESS, "100000000"],
            "type": "entry_function_payload"
        });

        assert_eq!(serde_json::from_str::<Value>(&result).unwrap(), expected);
    }

    #[test]
    fn test_build_unstake_payload() {
        let result = build_unstake_payload(TEST_DELEGATOR_ADDRESS, TEST_POOL_ADDRESS, "50000000");
        let expected = json!({
            "function": "0x1::delegation_pool::unlock",
            "type_arguments": [],
            "arguments": [TEST_POOL_ADDRESS, "50000000"],
            "type": "entry_function_payload"
        });

        assert_eq!(serde_json::from_str::<Value>(&result).unwrap(), expected);
    }

    #[test]
    fn test_build_withdraw_payload() {
        let result = build_withdraw_payload(TEST_DELEGATOR_ADDRESS, TEST_POOL_ADDRESS, "1102185008");
        let expected = json!({
            "function": "0x1::delegation_pool::withdraw",
            "type_arguments": [],
            "arguments": [TEST_POOL_ADDRESS, "1102185008"],
            "type": "entry_function_payload"
        });

        assert_eq!(serde_json::from_str::<Value>(&result).unwrap(), expected);
    }
}
