use crate::alien::{AlienProvider, AlienTarget};
use primitives::{ScanTransaction, ScanTransactionPayload};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GemApiClient {
    api_url: String,
    provider: Arc<dyn AlienProvider>,
}

impl GemApiClient {
    pub fn new(api_url: String, provider: Arc<dyn AlienProvider>) -> Self {
        Self { api_url, provider }
    }

    pub async fn scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, String> {
        let url = format!("{}/v1/scan/transaction", self.api_url);
        let target = AlienTarget::post_json(&url, &payload);
        let response = self.provider.request(target).await.map_err(|e| e.to_string())?;
        serde_json::from_slice(&response.data).map_err(|e| format!("Failed to parse response: {}", e))
    }
}
