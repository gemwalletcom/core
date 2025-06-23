// Real trace data from transaction: 0x825c8f677d215d4f128218aea1d9aa965d93790d8195f609ffb4fa6d4310fc79
// This is a Uniswap V4 swap transaction from ETH -> USDC -> USDT

use serde_json::json;

pub fn get_sample_trace_result() -> serde_json::Value {
    json!({
        "output": "0x675cae38",
        "stateDiff": {
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48": {
                "storage": {
                    "0x4d2c6926676a95578f1f83fc7206c731f15ec7c5dbd8d975202dea6c79518b66": {
                        "*": {
                            "from": "0x5b8b3c0e",
                            "to": "0x706035c"
                        }
                    }
                }
            },
            "0xdac17f958d2ee523a2206206994597c13d831ec7": {
                "storage": {
                    "0x5a78295ac94dacfc557f7106ea064b1acc0ac048c9e355c345613ff42cd4ff66": {
                        "*": {
                            "from": "0x0",
                            "to": "0x6586385"
                        }
                    }
                }
            }
        },
        "trace": [
            {
                "type": "call",
                "action": {
                    "from": "0x6bde9f8888e560adffdf14eb18a12ad96727e9c7",
                    "callType": "call", 
                    "gas": "0x2fa6bc4",
                    "input": "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006858b77d",
                    "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
                    "value": "0x0"
                },
                "error": "Reverted",
                "result": {
                    "gasUsed": "0x5f012",
                    "output": "0x675cae38"
                },
                "subtraces": 7,
                "traceAddress": []
            }
        ]
    })
}

#[allow(dead_code)]
pub fn get_sample_transaction_object() -> serde_json::Value {
    json!({
        "from": "0x6bde9f8888e560adffdf14eb18a12ad96727e9c7",
        "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
        "value": "0x0",
        "data": "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006858b77d"
    })
}

// Known token addresses and their balance slot indices
pub fn get_test_token_configs() -> Vec<(String, u32)> {
    vec![
        // USDC - slot 9
        ("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(), 9),
        // USDT - slot 2  
        ("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(), 2),
    ]
}

pub fn get_test_participants() -> Vec<String> {
    vec![
        "0x6bde9f8888e560adffdf14eb18a12ad96727e9c7".to_string(), // Transaction sender
        "0x66a9893cc07d91d95644aedd05d03f95e1dba8af".to_string(), // Uniswap V4 router  
        "0x27213e28d7fda5c57fe9e5dd923818dbccf71c47".to_string(), // Fee recipient
    ]
}

// Test data with ETH balance changes
pub fn get_trace_result_with_eth_deltas() -> serde_json::Value {
    json!({
        "output": "0x675cae38",
        "stateDiff": {
            "0x6bde9f8888e560adffdf14eb18a12ad96727e9c7": {
                "balance": {
                    "*": {
                        "from": "0x5af3107a4000", // ~100 ETH
                        "to": "0x5af3107a3000"   // ~99 ETH (paid gas)
                    }
                }
            },
            "0x66a9893cc07d91d95644aedd05d03f95e1dba8af": {
                "balance": {
                    "*": {
                        "from": "0x1000",
                        "to": "0x2000"
                    }
                }
            }
        }
    })
}