use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::result::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressTarget {
    pub address: String,
    pub chain: primitives::Chain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScanTarget {
    Address(AddressTarget),
    URL(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub target: ScanTarget,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub name: Option<String>,
    pub provider: String,
    pub verified: bool,
    pub required_memo: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub is_malicious: bool,
    pub target: ScanTarget,
    pub metadata: Option<Metadata>,
}

#[async_trait]
pub trait SecurityProvider: Send + Sync {
    fn new(url: &str, api_key: &str) -> Self
    where
        Self: Sized;

    fn name(&self) -> &'static str;
    async fn scan(&self, target: &ScanTarget) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>>;
}
