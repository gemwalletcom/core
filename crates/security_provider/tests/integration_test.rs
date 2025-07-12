use primitives::Chain;
use security_provider::{mapper, ScanResult, TokenTarget};

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

#[test]
fn test_chain_to_provider_id() {
    assert_eq!(mapper::chain_to_provider_id(Chain::Ethereum), "1");
    assert_eq!(mapper::chain_to_provider_id(Chain::SmartChain), "56");
    assert_eq!(mapper::chain_to_provider_id(Chain::Polygon), "137");
    assert_eq!(mapper::chain_to_provider_id(Chain::Arbitrum), "42161");
    assert_eq!(mapper::chain_to_provider_id(Chain::Optimism), "10");
    assert_eq!(mapper::chain_to_provider_id(Chain::Base), "8453");
    assert_eq!(mapper::chain_to_provider_id(Chain::Bitcoin), "1"); // Unsupported, defaults to Ethereum
}
