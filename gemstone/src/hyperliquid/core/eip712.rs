use alloy_primitives::Address;
use gem_evm::eip712::{eip712_domain_types, EIP712Type};
use primitives::Chain;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::models::PhantomAgent;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HyperLiquidEIP712Domain {
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    pub name: String,
    #[serde(rename = "verifyingContract")]
    pub verifying_contract: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HyperLiquidEIP712Message {
    pub domain: HyperLiquidEIP712Domain,
    pub message: Value,
    #[serde(rename = "primaryType")]
    pub primary_type: String,
    pub types: BTreeMap<String, Vec<EIP712Type>>,
}

pub fn create_l1_eip712_json(phantom_agent: &PhantomAgent) -> String {
    let domain = HyperLiquidEIP712Domain {
        chain_id: Chain::HyperCore.network_id().parse().unwrap(),
        name: "Exchange".to_string(),
        verifying_contract: Some(Address::ZERO.to_string()),
        version: Some("1".to_string()),
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

    let domain = HyperLiquidEIP712Domain {
        chain_id,
        name: "HyperliquidSignTransaction".to_string(),
        verifying_contract: Some(Address::ZERO.to_string()),
        version: Some("1".to_string()),
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
            name: "hyperliquidChain".to_string(),
            r#type: "string".to_string(),
        },
        EIP712Type {
            name: "destination".to_string(),
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
    ]
}

pub fn approve_agent_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "hyperliquidChain".to_string(),
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
            name: "hyperliquidChain".to_string(),
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
