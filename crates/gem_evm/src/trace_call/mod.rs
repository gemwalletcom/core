pub mod model;

#[cfg(test)]
mod test_data;

use std::collections::HashMap;
use serde_json::Value;
use num_bigint::BigInt;
use gem_hash::keccak::keccak256;
use serde_serializers::bigint::deserialize_bigint_from_str;

pub use model::*;

pub fn extract_deltas(
    result: TraceCallResult,
    tracked_tokens: &HashMap<Address, u32>,
    participants: &[Address],
) -> HashMap<Address, Delta> {
    let mut deltas: HashMap<Address, Delta> = HashMap::new();

    if let Some(state_diff) = result.state_diff {
        // Extract native ETH deltas
        for (addr, diff) in state_diff.iter() {
            if let Some(balance_changes) = &diff.balance {
                for (change_type, balance_change) in balance_changes {
                    if change_type == "*" {
                        if let (Ok(from_val), Ok(to_val)) = (
                            parse_hex_string_to_bigint(&balance_change.from),
                            parse_hex_string_to_bigint(&balance_change.to),
                        ) {
                            let delta_eth = &to_val - &from_val;
                            if delta_eth != BigInt::from(0) {
                                deltas
                                    .entry(addr.clone())
                                    .or_default()
                                    .eth = delta_eth;
                            }
                        }
                    }
                }
            }
        }

        // Extract ERC-20 token deltas
        for (token_addr, slot_idx) in tracked_tokens {
            if let Some(token_diff) = state_diff.get(token_addr) {
                if let Some(storage_changes) = &token_diff.storage {
                    for addr in participants {
                        let slot_key = compute_mapping_slot_key(addr, *slot_idx);
                        if let Some(storage_change) = storage_changes.get(&slot_key) {
                            match storage_change {
                                StorageChange::Change { change } => {
                                    if let (Ok(from_val), Ok(to_val)) = (
                                        parse_hex_string_to_bigint(&change.from),
                                        parse_hex_string_to_bigint(&change.to),
                                    ) {
                                        let delta_tok = &to_val - &from_val;
                                        if delta_tok != BigInt::from(0) {
                                            deltas
                                                .entry(addr.clone())
                                                .or_default()
                                                .tokens
                                                .insert(token_addr.clone(), delta_tok);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    deltas
}

fn parse_hex_string_to_bigint(hex_str: &str) -> Result<BigInt, String> {
    use serde::de::value::{Error as ValueError, StringDeserializer};
    use serde::de::IntoDeserializer;
    
    let deserializer: StringDeserializer<ValueError> = hex_str.to_string().into_deserializer();
    deserialize_bigint_from_str(deserializer).map_err(|e| e.to_string())
}

fn compute_mapping_slot_key(address: &str, slot_idx: u32) -> String {
    let addr_bytes = pad32_address(address);
    let slot_bytes = pad32_u32(slot_idx);
    let combined = [addr_bytes, slot_bytes].concat();
    let hash = keccak256(&combined);
    format!("0x{}", hex::encode(hash))
}

fn pad32_address(address: &str) -> Vec<u8> {
    let addr_str = address.strip_prefix("0x").unwrap_or(address);
    let mut addr_bytes = hex::decode(addr_str).expect("Invalid address hex");
    
    // Pad to 32 bytes
    let mut padded = vec![0u8; 32 - addr_bytes.len()];
    padded.append(&mut addr_bytes);
    padded
}

fn pad32_u32(value: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 28];
    bytes.extend_from_slice(&value.to_be_bytes());
    bytes
}

pub fn extract_deltas_from_trace_result(
    result: TraceCallResult,
    config: &DeltaExtractorConfig,
    participants: &[Address],
) -> EthereumDeltaResult {
    let tracked_tokens = config.get_token_slot_mapping();
    let deltas_map = extract_deltas(result, &tracked_tokens, participants);
    
    let mut ethereum_result = EthereumDeltaResult::new();
    
    for (address, delta) in deltas_map {
        let mut address_delta = AddressDelta::new(address);
        address_delta.eth_delta = delta.eth;
        address_delta.token_deltas = delta.tokens;
        
        ethereum_result.add_delta(address_delta);
    }
    
    ethereum_result
}

pub fn extract_addresses_from_trace(trace: &Value) -> Vec<Address> {
    let mut addresses = std::collections::HashSet::new();
    
    fn extract_from_trace_recursive(trace: &Value, addresses: &mut std::collections::HashSet<Address>) {
        if let Some(trace_array) = trace.as_array() {
            for trace_item in trace_array {
                extract_from_single_trace(trace_item, addresses);
            }
        } else if trace.is_object() {
            extract_from_single_trace(trace, addresses);
        }
    }
    
    fn extract_from_single_trace(trace: &Value, addresses: &mut std::collections::HashSet<Address>) {
        // Extract from action
        if let Some(action) = trace.get("action") {
            if let Some(from) = action.get("from").and_then(|v| v.as_str()) {
                addresses.insert(from.to_string());
            }
            if let Some(to) = action.get("to").and_then(|v| v.as_str()) {
                addresses.insert(to.to_string());
            }
            if let Some(address) = action.get("address").and_then(|v| v.as_str()) {
                addresses.insert(address.to_string());
            }
        }
        
        // Extract from result
        if let Some(result) = trace.get("result") {
            if let Some(address) = result.get("address").and_then(|v| v.as_str()) {
                addresses.insert(address.to_string());
            }
        }
        
        // Recursively process subtraces
        if let Some(subtraces) = trace.get("subtraces") {
            if subtraces.as_u64().unwrap_or(0) > 0 {
                // Look for nested traces in the parent structure
                // This is a simplified approach - real implementation might need more sophisticated parsing
            }
        }
    }
    
    extract_from_trace_recursive(trace, &mut addresses);
    addresses.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad32_address() {
        let addr = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        let padded = pad32_address(addr);
        assert_eq!(padded.len(), 32);
    }

    #[test]
    fn test_pad32_u32() {
        let value = 0u32;
        let padded = pad32_u32(value);
        assert_eq!(padded.len(), 32);
        assert_eq!(padded[31], 0);
    }

    #[test]
    fn test_parse_hex_string_to_bigint() {
        assert_eq!(parse_hex_string_to_bigint("0x0").unwrap(), BigInt::from(0));
        assert_eq!(parse_hex_string_to_bigint("0x1").unwrap(), BigInt::from(1));
        assert_eq!(parse_hex_string_to_bigint("0xff").unwrap(), BigInt::from(255));
        assert_eq!(parse_hex_string_to_bigint("123").unwrap(), BigInt::from(123)); // decimal
    }

    #[test]
    fn test_compute_mapping_slot_key() {
        let addr = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        let slot = 0u32;
        let key = compute_mapping_slot_key(addr, slot);
        assert!(key.starts_with("0x"));
        assert_eq!(key.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_extract_deltas_with_real_data() {
        use test_data::*;
        
        let trace_data = get_sample_trace_result();
        let trace_result: TraceCallResult = serde_json::from_value(trace_data).expect("Failed to parse trace result");
        
        let token_configs = get_test_token_configs();
        let tracked_tokens: HashMap<Address, u32> = token_configs.into_iter().collect();
        let participants = get_test_participants();
        
        let deltas = extract_deltas(trace_result, &tracked_tokens, &participants);
        
        // Should have extracted some deltas
        assert!(!deltas.is_empty(), "Should have extracted some deltas");
        
        // Check if we have deltas for expected addresses
        for participant in &participants {
            if let Some(delta) = deltas.get(participant) {
                println!("Address {}: ETH delta = {}, token deltas = {:?}", 
                    participant, delta.eth, delta.tokens);
            }
        }
    }

    #[test]
    fn test_extract_deltas_from_trace_result_integration() {
        use test_data::*;
        
        let trace_data = get_sample_trace_result();
        let trace_result: TraceCallResult = serde_json::from_value(trace_data).expect("Failed to parse trace result");
        
        let mut config = DeltaExtractorConfig::new();
        
        // Add known tokens from the transaction
        config = config.add_token(TokenConfig::new("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(), 9).with_metadata("USDC".to_string(), 6));
        config = config.add_token(TokenConfig::new("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(), 2).with_metadata("USDT".to_string(), 6));
        
        let participants = get_test_participants();
        
        let result = extract_deltas_from_trace_result(trace_result, &config, &participants);
        
        // Should have created a valid result
        assert!(result.transaction_hash.is_none()); // We didn't set it
        
        // Print out extracted deltas for manual verification
        for delta in &result.deltas {
            println!("Address {}: ETH Δ = {}, Token Δs = {:?}", 
                delta.address, delta.eth_delta, delta.token_deltas);
        }
        
        // Basic sanity checks
        assert!(result.deltas.len() <= participants.len(), "Should not have more deltas than participants");
    }

    #[test]
    fn test_usdc_balance_slot_computation() {
        // Test USDC balance slot computation for a known address
        let usdc_holder = "0x66a9893cc07d91d95644aedd05d03f95e1dba8af"; // From the trace
        let usdc_balance_slot = 9u32;
        
        let slot_key = compute_mapping_slot_key(usdc_holder, usdc_balance_slot);
        
        // This should match one of the storage keys in the stateDiff
        println!("USDC balance slot key for {}: {}", usdc_holder, slot_key);
        
        // Verify the format
        assert!(slot_key.starts_with("0x"));
        assert_eq!(slot_key.len(), 66);
    }

    #[test]
    fn test_usdt_balance_slot_computation() {
        // Test USDT balance slot computation for a known address
        let usdt_holder = "0x66a9893cc07d91d95644aedd05d03f95e1dba8af"; // From the trace
        let usdt_balance_slot = 2u32;
        
        let slot_key = compute_mapping_slot_key(usdt_holder, usdt_balance_slot);
        
        // This should match one of the storage keys in the stateDiff
        println!("USDT balance slot key for {}: {}", usdt_holder, slot_key);
        
        // Verify the format
        assert!(slot_key.starts_with("0x"));
        assert_eq!(slot_key.len(), 66);
    }

    #[test]
    fn test_state_diff_parsing() {
        use test_data::*;
        
        let trace_data = get_sample_trace_result();
        
        // Extract just the stateDiff for focused testing
        if let Some(state_diff_value) = trace_data.get("stateDiff") {
            let state_diff: HashMap<Address, StateDiff> = serde_json::from_value(state_diff_value.clone())
                .expect("Failed to parse stateDiff");
            
            println!("Found {} addresses in stateDiff", state_diff.len());
            
            for (addr, diff) in &state_diff {
                println!("Address: {}", addr);
                
                if let Some(storage) = &diff.storage {
                    println!("  Storage changes: {}", storage.len());
                    for (slot, change) in storage {
                        match change {
                            StorageChange::Change { change } => {
                                println!("    Slot {}: {} -> {}", slot, change.from, change.to);
                            }
                        }
                    }
                }
                
                if let Some(balance_changes) = &diff.balance {
                    for (change_type, balance_change) in balance_changes {
                        println!("  Balance {}: {} -> {}", change_type, balance_change.from, balance_change.to);
                    }
                }
            }
        }
    }

    #[test]
    fn test_eth_balance_extraction() {
        use test_data::*;
        
        let trace_data = get_trace_result_with_eth_deltas();
        let trace_result: TraceCallResult = serde_json::from_value(trace_data).expect("Failed to parse trace result");
        
        let tracked_tokens = HashMap::new(); // No token tracking for this test
        let participants = vec![
            "0x6bde9f8888e560adffdf14eb18a12ad96727e9c7".to_string(),
            "0x66a9893cc07d91d95644aedd05d03f95e1dba8af".to_string(),
        ];
        
        let deltas = extract_deltas(trace_result, &tracked_tokens, &participants);
        
        // Should have extracted ETH deltas for both addresses
        assert_eq!(deltas.len(), 2, "Should have deltas for 2 addresses");
        
        // Check first address (sender) - should have negative ETH delta (paid gas)
        let sender_delta = deltas.get("0x6bde9f8888e560adffdf14eb18a12ad96727e9c7").unwrap();
        assert!(sender_delta.eth < BigInt::from(0), "Sender should have negative ETH delta");
        println!("Sender ETH delta: {}", sender_delta.eth);
        
        // Check second address (router) - should have positive ETH delta
        let router_delta = deltas.get("0x66a9893cc07d91d95644aedd05d03f95e1dba8af").unwrap();
        assert!(router_delta.eth > BigInt::from(0), "Router should have positive ETH delta");
        println!("Router ETH delta: {}", router_delta.eth);
    }

    #[test] 
    fn test_token_balance_extraction() {
        use test_data::*;
        
        let trace_data = get_sample_trace_result();
        let trace_result: TraceCallResult = serde_json::from_value(trace_data).expect("Failed to parse trace result");
        
        let token_configs = get_test_token_configs();
        let tracked_tokens: HashMap<Address, u32> = token_configs.into_iter().collect();
        let participants = get_test_participants();
        
        let deltas = extract_deltas(trace_result, &tracked_tokens, &participants);
        
        println!("Extracted deltas for {} addresses", deltas.len());
        
        for (addr, delta) in &deltas {
            println!("Address {}: ETH Δ = {}, Token Δs = {:?}", addr, delta.eth, delta.tokens);
            
            // Verify token deltas are reasonable
            for (token_addr, token_delta) in &delta.tokens {
                assert!(token_delta != &BigInt::from(0), "Token delta should not be zero");
                println!("  Token {} delta: {}", token_addr, token_delta);
            }
        }
    }

    #[test]
    fn test_extract_addresses_from_trace() {
        use test_data::*;
        
        let trace_data = get_sample_trace_result();
        if let Some(trace) = trace_data.get("trace") {
            let addresses = extract_addresses_from_trace(trace);
            
            println!("Extracted {} addresses from trace", addresses.len());
            for addr in &addresses {
                println!("  Address: {}", addr);
            }
            
            // Should have extracted at least the sender and router addresses
            assert!(!addresses.is_empty(), "Should extract some addresses from trace");
            assert!(addresses.contains(&"0x6bde9f8888e560adffdf14eb18a12ad96727e9c7".to_string()), "Should contain sender address");
            assert!(addresses.contains(&"0x66a9893cc07d91d95644aedd05d03f95e1dba8af".to_string()), "Should contain router address");
        }
    }
}

#[cfg(all(test, feature = "rpc"))]
mod examples {
    use super::*;
    
    /// Example demonstrating the typed trace_call API usage
    #[allow(dead_code)]
    async fn example_typed_trace_call_usage() {
        use alloy_rpc_types::TransactionRequest;
        use crate::rpc::client::EthereumClient;
        use primitives::EVMChain;
        
        // Setup client
        let client = EthereumClient::new(EVMChain::Ethereum, "https://eth.llamarpc.com".to_string());
        
        // Create transaction request  
        let tx_request = TransactionRequest {
            from: Some("0x6bde9f8888e560adffdf14eb18a12ad96727e9c7".parse().unwrap()),
            to: Some("0x66a9893cc07d91d95644aedd05d03f95e1dba8af".parse().into()),
            value: Some(0.into()),
            input: Some("0x3593564c".parse().unwrap().into()),
            ..Default::default()
        };
        
        // Make typed trace call - no manual JSON parsing needed!
        let trace_result: TraceCallResult = client
            .trace_call(tx_request, vec!["trace".to_string(), "stateDiff".to_string()])
            .await
            .expect("trace_call failed");
        
        // Direct access to typed fields
        if let Some(state_diff) = trace_result.state_diff {
            println!("Found state changes for {} addresses", state_diff.len());
            
            for (address, diff) in state_diff {
                if let Some(balance_changes) = diff.balance {
                    println!("Address {} has balance changes", address);
                }
                
                if let Some(storage_changes) = diff.storage {
                    println!("Address {} has {} storage changes", address, storage_changes.len());
                }
            }
        }
        
        // Extract participant addresses from trace
        if let Some(trace) = trace_result.trace {
            let participants = extract_addresses_from_trace(&trace);
            println!("Found {} participant addresses", participants.len());
        }
        
        // Setup token tracking config
        let config = DeltaExtractorConfig::new()
            .add_token(TokenConfig::new("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(), 9)
                .with_metadata("USDC".to_string(), 6))
            .add_token(TokenConfig::new("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(), 2)
                .with_metadata("USDT".to_string(), 6));
        
        // Extract deltas using the typed result
        let participants = vec!["0x6bde9f8888e560adffdf14eb18a12ad96727e9c7".to_string()];
        let delta_result = extract_deltas_from_trace_result(trace_result, &config, &participants);
        
        // Process results with type safety
        for delta in delta_result.deltas {
            println!("Address {}: ETH Δ = {}", delta.address, delta.eth_delta);
            for (token_addr, token_delta) in delta.token_deltas {
                println!("  Token {} Δ = {}", token_addr, token_delta);
            }
        }
    }
}