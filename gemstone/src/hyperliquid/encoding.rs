use alloy_primitives::{hex, Address};
use gem_evm::eip712::{eip712_domain_types, EIP712Domain, EIP712Type};
use gem_hash::keccak::keccak256;
use primitives::Chain;
use rmp_serde;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HyperLiquidEIP712Message {
    pub domain: EIP712Domain,
    pub message: Value,
    #[serde(rename = "primaryType")]
    pub primary_type: String,
    pub types: BTreeMap<String, Vec<EIP712Type>>,
}

pub fn create_l1_eip712_json(phantom_agent: &PhantomAgent) -> String {
    let domain = EIP712Domain {
        name: "Exchange".to_string(),
        version: Some("1".to_string()),
        chain_id: Chain::HyperCore.network_id().parse().unwrap(),
        verifying_contract: Some(Address::ZERO.to_string()),
        salts: None,
    };

    let message = serde_json::to_value(phantom_agent).unwrap();

    let mut types = BTreeMap::new();
    types.insert("EIP712Domain".to_string(), eip712_domain_types());
    types.insert("Agent".to_string(), agent_types());

    let eip712_message = HyperLiquidEIP712Message {
        domain,
        message,
        primary_type: "Agent".to_string(),
        types,
    };

    serde_json::to_string_pretty(&eip712_message).unwrap()
}

pub fn create_user_signed_eip712_json(action: &Value, primary_type: &str, action_types: Vec<EIP712Type>) -> String {
    let arbitrum_chain_id: u64 = Chain::Arbitrum.network_id().parse().unwrap();
    let chain_id = if let Some(sig_chain_id) = action.get("signatureChainId").and_then(|v| v.as_str()) {
        u64::from_str_radix(sig_chain_id.trim_start_matches("0x"), 16).unwrap_or(arbitrum_chain_id)
    } else {
        arbitrum_chain_id
    };

    let domain = EIP712Domain {
        name: "HyperliquidSignTransaction".to_string(),
        version: Some("1".to_string()),
        chain_id,
        verifying_contract: Some(Address::ZERO.to_string()),
        salts: None,
    };

    let mut types = BTreeMap::new();
    types.insert("EIP712Domain".to_string(), eip712_domain_types());
    types.insert(primary_type.to_string(), action_types);

    let eip712_message = HyperLiquidEIP712Message {
        domain,
        message: action.clone(),
        primary_type: primary_type.to_string(),
        types,
    };

    serde_json::to_string_pretty(&eip712_message).unwrap()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhantomAgent {
    pub source: String,
    #[serde(rename = "connectionId")]
    pub connection_id: String,
}

impl PhantomAgent {
    pub fn new(action_hash: String) -> Self {
        Self {
            source: "a".to_string(),
            connection_id: format!("0x{action_hash}"),
        }
    }
}

pub fn action_hash(action: &Value, vault_address: Option<&str>, nonce: u64, expires_after: Option<u64>) -> Result<String, String> {
    // Serialize action with msgpack
    let mut data = rmp_serde::to_vec(action).map_err(|e| format!("Failed to serialize action: {e}"))?;

    // Add nonce (8 bytes, big endian)
    data.extend_from_slice(&nonce.to_be_bytes());

    // Handle vault address
    if let Some(vault) = vault_address {
        data.push(0x01);
        // Parse vault address and add as bytes
        let vault_bytes = hex::decode(vault.trim_start_matches("0x")).map_err(|e| format!("Invalid vault address: {e}"))?;
        data.extend_from_slice(&vault_bytes);
    } else {
        data.push(0x00);
    }

    // Handle expiration
    if let Some(expires) = expires_after {
        data.push(0x00);
        data.extend_from_slice(&expires.to_be_bytes());
    } else {
        data.push(0x01);
    }

    // Calculate keccak256 hash
    let hash = keccak256(&data);
    Ok(hex::encode(hash))
}

// Helper functions for HyperLiquid-specific EIP712 type definitions
pub fn agent_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "source".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "connectionId".to_string(),
            r#type: "bytes32".to_string(),
        },
    ]
}

pub fn withdraw_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "type".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "hyperliquidChain".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "signatureChainId".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "amount".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "time".to_string(),
            r#type: "uint64".to_string(),
        },
        EIP712Type {
            name: "destination".to_string(),
            r#type: "address".to_string(),
        },
    ]
}

pub fn approve_agent_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "type".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "hyperliquidChain".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "signatureChainId".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "agentAddress".to_string(),
            r#type: "address".to_string(),
        },
        EIP712Type {
            name: "agentName".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "nonce".to_string(),
            r#type: "uint64".to_string(),
        },
    ]
}

pub fn approve_builder_fee_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "type".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "hyperliquidChain".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "signatureChainId".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "maxFeeRate".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "builder".to_string(),
            r#type: "address".to_string(),
        },
        EIP712Type {
            name: "nonce".to_string(),
            r#type: "uint64".to_string(),
        },
    ]
}

pub fn set_referrer_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "type".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "code".to_string(),
            r#type: "string".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_key_ordering_preserved() {
        // Test that JSON keys are consistently ordered with preserve_order feature
        let phantom_agent = PhantomAgent {
            source: "a".to_string(),
            connection_id: "0x1234567890abcdef".to_string(),
        };

        // Generate JSON multiple times
        let json1 = create_l1_eip712_json(&phantom_agent);
        let json2 = create_l1_eip712_json(&phantom_agent);

        // Should be identical (keys in same order)
        assert_eq!(json1, json2);

        // Check that keys appear in expected order for domain
        assert!(json1.find("\"domain\"").unwrap() < json1.find("\"message\"").unwrap());
        assert!(json1.find("\"message\"").unwrap() < json1.find("\"primaryType\"").unwrap());
        assert!(json1.find("\"primaryType\"").unwrap() < json1.find("\"types\"").unwrap());
    }
}
