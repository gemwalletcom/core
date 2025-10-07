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

pub fn build_withdraw_payload(_delegator_address: &str, pool_address: &str) -> String {
    let payload = json!({
        "function": "0x1::delegation_pool::withdraw",
        "type_arguments": [],
        "arguments": [pool_address, "0"],
        "type": "entry_function_payload"
    });
    payload.to_string()
}
