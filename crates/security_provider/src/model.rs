use primitives::Chain;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressTarget {
    pub address: String,
    pub chain: Chain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTarget {
    pub token_id: String,
    pub chain: Chain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum ScanTarget {
    Address(AddressTarget),
    Token(TokenTarget),
    URL(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub target: ScanTarget,
    #[serde(rename = "type")]
    pub target_type: ScanTargetType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScanTargetType {
    Address,
    Token,
    URL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult<T> {
    pub target: T,
    pub is_malicious: bool,
    pub reason: Option<String>,
    pub provider: String,
}

#[test]
fn test_token_target_serialization() {
    let target = TokenTarget {
        token_id: "0xa0b86a33e6776a8e5b01b22e54e12b5e5d0f96f8".to_string(),
        chain: Chain::Ethereum,
    };

    let serialized = serde_json::to_string(&target).unwrap();
    let deserialized: TokenTarget = serde_json::from_str(&serialized).unwrap();

    assert_eq!(target.token_id, deserialized.token_id);
    assert_eq!(target.chain, deserialized.chain);
}

#[test]
fn test_scan_result_token_target() {
    let target = TokenTarget {
        token_id: "0xa0b86a33e6776a8e5b01b22e54e12b5e5d0f96f8".to_string(),
        chain: Chain::Ethereum,
    };

    let result = ScanResult {
        target: target.clone(),
        is_malicious: true,
        reason: Some("Test reason".to_string()),
        provider: "test_provider".to_string(),
    };

    assert_eq!(result.target.token_id, target.token_id);
    assert_eq!(result.target.chain, target.chain);
    assert!(result.is_malicious);
    assert_eq!(result.reason, Some("Test reason".to_string()));
    assert_eq!(result.provider, "test_provider");
}
