use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::result::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressTarget {
    pub address: String,
    pub chain: primitives::Chain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum ScanTarget {
    Address(AddressTarget),
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
    URL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub name: Option<String>,
    pub verified: bool,
    pub required_memo: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub is_malicious: bool,
    pub metadata: Option<Metadata>,
    pub provider: String,
}

#[async_trait]
pub trait SecurityProvider: Send + Sync {
    fn new(url: &str, api_key: &str) -> Self
    where
        Self: Sized;

    fn name(&self) -> &'static str;
    async fn scan(&self, target: &ScanTarget) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_address_target() {
        let address_target = AddressTarget {
            address: "0x1234567890abcdef".to_string(),
            chain: primitives::Chain::Ethereum,
        };
        let target = ScanTarget::Address(address_target);
        let request = ScanRequest {
            target,
            target_type: ScanTargetType::Address,
        };

        let json = serde_json::to_string(&request).unwrap();
        let expected = r#"{"target":{"address":"0x1234567890abcdef","chain":"ethereum"},"type":"address"}"#;

        assert_eq!(json, expected);
    }

    #[test]
    fn test_url_target() {
        let url_target = "https://example.com".to_string();
        let target = ScanTarget::URL(url_target);
        let request = ScanRequest {
            target,
            target_type: ScanTargetType::URL,
        };

        let json = serde_json::to_string(&request).unwrap();
        let expected = r#"{"target":"https://example.com","type":"url"}"#;

        assert_eq!(json, expected);
    }
}
