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
pub struct ScanResult<T> {
    pub target: T,
    pub is_malicious: bool,
    pub reason: Option<String>,
    pub provider: String,
}

#[async_trait]
pub trait ScanProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn std::error::Error + Send + Sync>>;
    async fn scan_url(&self, target: &str) -> Result<ScanResult<String>, Box<dyn std::error::Error + Send + Sync>>;
}
