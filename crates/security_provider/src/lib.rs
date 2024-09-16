use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub target: String,
    pub target_type: String, // "url" or "contract"
}

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub is_malicious: bool,
    pub risk_score: u8,
    pub details: String,
}

#[async_trait]
pub trait SecurityProvider: Send + Sync {
    fn new(url: &str, api_key: &str) -> Self
    where
        Self: Sized;
    async fn scan(&self, target: &str, target_type: &str) -> ScanResult;
}
